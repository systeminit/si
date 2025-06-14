use std::{
    marker::PhantomData,
    net::{
        SocketAddr,
        ToSocketAddrs,
    },
    path::PathBuf,
    result,
    str::{
        self,
        FromStr,
    },
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use cyclone_core::{
    CycloneRequest,
    CycloneRequestable,
    LivenessStatus,
    LivenessStatusParseError,
    ReadinessStatus,
    ReadinessStatusParseError,
};
use http::{
    request::Builder,
    uri::{
        Authority,
        InvalidUri,
        InvalidUriParts,
        PathAndQuery,
        Scheme,
    },
};
use hyper::{
    Body,
    Method,
    Request,
    Response,
    StatusCode,
    Uri,
    body,
    client::{
        HttpConnector,
        ResponseFuture,
        connect::Connection,
    },
    service::Service,
};
use hyperlocal::{
    UnixClientExt,
    UnixConnector,
    UnixStream,
};
use telemetry::prelude::*;
use telemetry_http::propagation;
use thiserror::Error;
use tokio::{
    io::{
        AsyncRead,
        AsyncReadExt,
        AsyncWrite,
        AsyncWriteExt,
    },
    net::TcpStream,
};
use tokio_tungstenite::{
    WebSocketStream,
    tungstenite::{
        client::IntoClientRequest,
        handshake::client::Request as WsRequest,
    },
};

use crate::{
    Execution,
    PingExecution,
    Watch,
    new_unstarted_execution,
    ping,
    watch,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("cannot create client uri")]
    ClientUri(#[source] http::Error),
    #[error("failed to connect")]
    Connect(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to connect to the Firecracker VM")]
    FirecrackerConnect,
    #[error("invalid liveness status")]
    InvalidLivenessStatus(#[from] LivenessStatusParseError),
    #[error("invalid readiness status")]
    InvalidReadinessStatus(#[from] ReadinessStatusParseError),
    #[error("invalid URI")]
    InvalidUri(#[from] InvalidUri),
    #[error("invalid websocket uri scheme: {0}")]
    InvalidWebsocketScheme(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("missing authority")]
    MissingAuthority,
    #[error("missing websocket scheme")]
    MissingWebsocketScheme,
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error("failed reading http response body")]
    ReadResponseBody(#[source] hyper::Error),
    #[error("failed to create an http request")]
    Request(#[source] hyper::http::Error),
    #[error("failed to create request uri")]
    RequestUri(#[source] InvalidUriParts),
    #[error("http response failed")]
    Response(#[source] hyper::Error),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
    #[error("failed to create a tungstenite http request")]
    TungsteniteRequest(#[source] tokio_tungstenite::tungstenite::Error),
    #[error("unexpected status code: {0}")]
    UnexpectedStatusCode(StatusCode),
    #[error("client is not healthy")]
    Unhealthy(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("failed to decode as a UTF8 string")]
    Utf8Decode(#[from] std::str::Utf8Error),
    #[error("failed to establish a websocket connection")]
    WebsocketConnection(#[source] tokio_tungstenite::tungstenite::Error),
}

impl ClientError {
    pub fn unhealthy(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unhealthy(Box::new(source))
    }
}

type Result<T> = result::Result<T, ClientError>;

#[derive(Debug)]
pub struct Client<Conn, Strm, Sock> {
    config: Arc<ClientConfig>,
    inner_client: hyper::Client<Conn, Body>,
    connector: Conn,
    socket: Sock,
    uri: Uri,
    _phantom: PhantomData<Strm>,
}

impl<Conn, Strm, Sock> Clone for Client<Conn, Strm, Sock>
where
    Conn: Clone,
    Sock: Clone,
{
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            inner_client: self.inner_client.clone(),
            connector: self.connector.clone(),
            socket: self.socket.clone(),
            uri: self.uri.clone(),
            _phantom: PhantomData,
        }
    }
}

pub type UdsClient = Client<UnixConnector, UnixStream, PathBuf>;
pub type HttpClient = Client<HttpConnector, TcpStream, SocketAddr>;

#[async_trait]
pub trait CycloneClient<Strm>
where
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    async fn watch(&mut self) -> result::Result<Watch<Strm>, ClientError>;

    async fn liveness(&mut self) -> result::Result<LivenessStatus, ClientError>;

    async fn readiness(&mut self) -> result::Result<ReadinessStatus, ClientError>;

    async fn execute_ping(&mut self) -> result::Result<PingExecution<Strm>, ClientError>;

    async fn prepare_execution<Request>(
        &mut self,
        request: CycloneRequest<Request>,
    ) -> result::Result<Execution<Strm, Request, Request::Response>, ClientError>
    where
        Request: CycloneRequestable + Send + Sync;
}

impl Client<(), (), ()> {
    pub fn http(
        socket_addrs: impl ToSocketAddrs,
    ) -> Result<Client<HttpConnector, TcpStream, SocketAddr>> {
        let socket = socket_addrs
            .to_socket_addrs()
            .map_err(ClientError::SocketAddrResolve)?
            .next()
            .ok_or(ClientError::NoSocketAddrResolved)?;
        let connector = HttpConnector::new();
        let inner_client = hyper::Client::builder().build(connector.clone());
        let scheme = Scheme::HTTP;
        let authority = Authority::try_from(format!("{}:{}", socket.ip(), socket.port()))?;
        let uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query("/")
            .build()
            .map_err(ClientError::ClientUri)?;
        let config = Arc::new(ClientConfig::default());

        Ok(Client {
            config,
            inner_client,
            connector,
            socket,
            uri,
            _phantom: PhantomData,
        })
    }

    // TODO(scott): firecracker connect here feels really flimsy. This likely needs an enum to
    // select behavior over.
    pub fn uds(
        socket: impl Into<PathBuf>,
        config: Arc<ClientConfig>,
    ) -> Result<Client<UnixConnector, UnixStream, PathBuf>> {
        let socket = socket.into();
        let connector = UnixConnector;
        let inner_client = hyper::Client::unix();
        let scheme = Scheme::try_from("unix")?;
        let authority = Uri::from(hyperlocal::Uri::new(&socket, "/"))
            .into_parts()
            .authority
            .ok_or(ClientError::MissingAuthority)?;
        let uri = Uri::builder()
            .scheme(scheme)
            .authority(authority)
            .path_and_query("/")
            .build()
            .map_err(ClientError::ClientUri)?;

        Ok(Client {
            config,
            inner_client,
            connector,
            socket,
            uri,
            _phantom: PhantomData,
        })
    }
}

#[async_trait]
impl<Conn, Strm, Sock> CycloneClient<Strm> for Client<Conn, Strm, Sock>
where
    Conn: Service<Uri, Response = Strm> + Clone + Send + Sync + 'static,
    Conn::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Conn::Future: Unpin + Send,
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + Sync + 'static,
    Sock: Send + Sync + std::fmt::Debug,
{
    async fn watch(&mut self) -> Result<Watch<Strm>> {
        let stream = self.websocket_stream("/watch").await?;
        Ok(watch::watch(stream, self.config.watch_timeout))
    }

    async fn liveness(&mut self) -> Result<LivenessStatus> {
        let response = self.get("/liveness").await?;

        if response.status() != StatusCode::OK {
            return Err(ClientError::UnexpectedStatusCode(response.status()));
        }
        let body = body::to_bytes(response)
            .await
            .map_err(ClientError::ReadResponseBody)?;
        let result = LivenessStatus::from_str(str::from_utf8(body.as_ref())?)?;

        Ok(result)
    }

    async fn readiness(&mut self) -> Result<ReadinessStatus> {
        let response = self.get("/readiness").await?;

        if response.status() != StatusCode::OK {
            return Err(ClientError::UnexpectedStatusCode(response.status()));
        }
        let body = body::to_bytes(response)
            .await
            .map_err(ClientError::ReadResponseBody)?;
        let result = ReadinessStatus::from_str(str::from_utf8(body.as_ref())?)?;

        Ok(result)
    }

    async fn execute_ping(&mut self) -> Result<PingExecution<Strm>> {
        let stream = self.websocket_stream("/execute/ping").await?;
        Ok(ping::execute(stream))
    }

    async fn prepare_execution<Request>(
        &mut self,
        request: CycloneRequest<Request>,
    ) -> result::Result<Execution<Strm, Request, Request::Response>, ClientError>
    where
        Request: CycloneRequestable + Send + Sync,
    {
        let stream = self.websocket_stream(request.websocket_path()).await?;
        Ok(new_unstarted_execution(stream, request))
    }
}

impl<Conn, Strm, Sock> Client<Conn, Strm, Sock>
where
    Conn: Service<Uri, Response = Strm> + Clone + Send + Sync + 'static,
    Conn::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Conn::Future: Unpin + Send,
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + Sync + 'static,
    Sock: Send + Sync + std::fmt::Debug,
{
    fn http_request_uri<P>(&self, path_and_query: P) -> Result<Uri>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let mut parts = self.uri.clone().into_parts();
        parts.path_and_query = Some(path_and_query.try_into()?);
        let uri = Uri::from_parts(parts).map_err(ClientError::RequestUri)?;

        Ok(uri)
    }

    fn ws_request_uri<P>(&self, path_and_query: P) -> Result<Uri>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let mut parts = self.uri.clone().into_parts();
        let uri_scheme = parts.scheme.take();
        match uri_scheme {
            Some(scheme) => match scheme.as_str() {
                "http" | "unix" => {
                    let _replaced = parts.scheme.replace(Scheme::try_from("ws")?);
                }
                "https" => {
                    let _replaced = parts.scheme.replace(Scheme::try_from("wss")?);
                }
                unsupported => {
                    return Err(ClientError::InvalidWebsocketScheme(unsupported.to_string()));
                }
            },
            None => return Err(ClientError::MissingWebsocketScheme),
        }
        parts.path_and_query = Some(path_and_query.try_into()?);
        let uri = Uri::from_parts(parts).map_err(ClientError::RequestUri)?;

        Ok(uri)
    }

    fn new_http_request<P>(&self, path_and_query: P) -> Result<Builder>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let uri = self.http_request_uri(path_and_query)?;

        Ok(Request::builder().uri(uri))
    }

    fn new_ws_request<P>(&self, path_and_query: P) -> Result<WsRequest>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let uri = self.ws_request_uri(path_and_query)?;

        let mut request = uri
            .into_client_request()
            .map_err(ClientError::TungsteniteRequest)?;
        propagation::inject_headers(request.headers_mut());

        Ok(request)
    }

    async fn get<P>(&self, path_and_query: P) -> Result<Response<Body>>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let mut builder = self.new_http_request(path_and_query)?.method(Method::GET);
        match builder.headers_mut() {
            Some(headers) => propagation::inject_headers(headers),
            None => trace!("request builder has an error"),
        };
        let request = builder.body(Body::empty()).map_err(ClientError::Request)?;
        self.request(request).await.map_err(ClientError::Response)
    }

    fn request(&self, req: Request<Body>) -> ResponseFuture {
        self.inner_client.request(req)
    }

    async fn connect(&mut self) -> Result<Strm> {
        let mut stream = self
            .connector
            .call(self.uri.clone())
            .await
            .map_err(|err| ClientError::Connect(err.into()))?;

        // Firecracker requires a special connection method to be inserted at the head of the
        // stream.
        if self.config.firecracker_connect {
            let connect_cmd = "CONNECT 52\n";
            let mut retries = 30;
            let mut single_byte = vec![0u8; 1];

            stream.write_all(connect_cmd.as_bytes()).await?;

            loop {
                // We need to read off the response to clear the stream, but sometimes this connect
                // message hangs if the VM is still being allocated when we ask.
                stream = match tokio::time::timeout(
                    self.config.connect_timeout,
                    stream.read_exact(&mut single_byte),
                )
                .await
                {
                    Ok(_) => {
                        if single_byte == [b'\n'] || single_byte == [b'\0'] {
                            break;
                        };
                        stream
                    }
                    Err(_) => {
                        // We timed out, let's get a new stream and try again.
                        retries -= 1;
                        stream.shutdown().await?;
                        stream = self
                            .connector
                            .call(self.uri.clone())
                            .await
                            .map_err(|err| ClientError::Connect(err.into()))?;
                        stream.write_all(connect_cmd.as_bytes()).await?;
                        stream
                    }
                };

                if retries <= 0 {
                    return Err(ClientError::FirecrackerConnect);
                }
            }
        }
        Ok(stream)
    }

    pub async fn websocket_stream<P>(&mut self, path_and_query: P) -> Result<WebSocketStream<Strm>>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let stream = self.connect().await?;

        let request = self.new_ws_request(path_and_query)?;
        let (websocket_stream, response) = tokio_tungstenite::client_async(request, stream)
            .await
            .map_err(ClientError::WebsocketConnection)?;
        if response.status() != StatusCode::SWITCHING_PROTOCOLS {
            return Err(ClientError::UnexpectedStatusCode(response.status()));
        }

        Ok(websocket_stream)
    }
}

#[derive(Debug)]
pub struct ClientConfig {
    pub connect_timeout: Duration,
    pub firecracker_connect: bool,
    pub watch_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_millis(10),
            // firecracker-setup: change firecracker_connect to "true"
            firecracker_connect: false,
            watch_timeout: Duration::from_secs(10),
        }
    }
}

#[allow(clippy::panic, clippy::assertions_on_constants)]
#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        env,
        path::Path,
    };

    use base64::{
        Engine,
        engine::general_purpose,
    };
    use buck2_resources::Buck2Resources;
    use cyclone_core::{
        ActionRunRequest,
        ComponentKind,
        ComponentView,
        ComponentViewWithGeometry,
        FunctionResult,
        ManagementRequest,
        ProgressMessage,
        ResolverFunctionComponent,
        ResolverFunctionRequest,
        SchemaVariantDefinitionRequest,
        ValidationRequest,
    };
    use cyclone_server::{
        Config,
        ConfigBuilder,
        Runnable as _,
        Server,
    };
    use futures::StreamExt;
    use serde_json::json;
    use tempfile::{
        NamedTempFile,
        TempPath,
    };
    use test_log::test;
    use tracing::warn;

    use super::*;

    fn rand_uds() -> TempPath {
        NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path()
    }

    #[allow(clippy::disallowed_methods)] // Used to determine if running in development
    fn lang_server_path() -> String {
        if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
            let resources = Buck2Resources::read().expect("failed to read buck2 resources");

            let lang_server_cmd_path = resources
                .get_ends_with("lang-js")
                .expect("failed to get lang-js resource")
                .to_string_lossy()
                .to_string();

            warn!(
                lang_server_cmd_path = lang_server_cmd_path.as_str(),
                "detected development run",
            );

            lang_server_cmd_path
        } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
            let lang_server_cmd_path = Path::new(&dir)
                .join("../../bin/lang-js/target/lang-js")
                .canonicalize()
                .expect(
                    "failed to canonicalize local dev build of <root>/bin/lang-js/target/lang-js",
                )
                .to_string_lossy()
                .to_string();

            warn!(
                lang_server_cmd_path = lang_server_cmd_path.as_str(),
                "detected development run",
            );

            lang_server_cmd_path
        } else {
            unimplemented!("tests must be run either with Cargo or Buck2");
        }
    }

    async fn uds_server(builder: &mut ConfigBuilder, tmp_socket: &TempPath) -> Server {
        let config = builder
            .unix_domain_socket(tmp_socket)
            .try_lang_server_path(lang_server_path())
            .expect("failed to resolve lang server path")
            .build()
            .expect("failed to build config");

        Server::from_config(config, Box::new(telemetry::NoopClient))
            .await
            .expect("failed to init server")
    }

    async fn uds_client_for_running_server(
        builder: &mut ConfigBuilder,
        tmp_socket: &TempPath,
    ) -> UdsClient {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { env::set_var("SI_LANG_JS_LOG", "debug") };
        let server = uds_server(builder, tmp_socket).await;
        let path = server
            .local_socket()
            .as_domain_socket()
            .expect("expected a domain socket")
            .to_owned();
        tokio::spawn(async move { server.run().await });
        let config = Arc::new(ClientConfig::default());

        Client::uds(path, config).expect("failed to create uds client")
    }

    async fn http_server(builder: &mut ConfigBuilder) -> Server {
        let config = builder
            .http_socket("127.0.0.1:0")
            .expect("failed to resolve socket addr")
            .try_lang_server_path(lang_server_path())
            .expect("failed to resolve lang server path")
            .build()
            .expect("failed to build config");

        Server::from_config(config, Box::new(telemetry::NoopClient))
            .await
            .expect("failed to init server")
    }

    async fn http_client_for_running_server(builder: &mut ConfigBuilder) -> HttpClient {
        // TODO: Audit that the environment access only happens in single-threaded code.
        unsafe { env::set_var("SI_LANG_JS_LOG", "debug") };
        let server = http_server(builder).await;
        let socket = *server
            .local_socket()
            .as_socket_addr()
            .expect("expected a socket addr");
        tokio::spawn(async move { server.run().await });

        Client::http(socket).expect("failed to create client")
    }

    fn base64_encode(input: impl AsRef<[u8]>) -> String {
        general_purpose::STANDARD_NO_PAD.encode(input)
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_watch() {
        let mut builder = Config::builder();
        let mut client =
            http_client_for_running_server(builder.watch(Some(Duration::from_secs(2)))).await;

        // Start the protocol
        let mut progress = client
            .watch()
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume 3 pings
        for _ in 0..2 {
            match progress.next().await {
                Some(Ok(_)) => assert!(true),
                Some(Err(err)) => panic!("failed to receive ping; err={err:?}"),
                None => panic!("stream ended early"),
            }
        }

        // Signal the client's desire to stop the watch
        progress.stop().await.expect("failed to stop protocol");
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_watch() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.watch(Some(Duration::from_secs(2))), &tmp_socket)
                .await;

        // Start the protocol
        let mut progress = client
            .watch()
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume 3 pings
        for _ in 0..2 {
            match progress.next().await {
                Some(Ok(_)) => assert!(true),
                Some(Err(err)) => panic!("failed to receive ping; err={err:?}"),
                None => panic!("stream ended early"),
            }
        }

        // Signal the client's desire to stop the watch
        progress.stop().await.expect("failed to stop protocol");
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_liveness() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_liveness() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_readiness() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_readiness() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_execute_ping() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(true)).await;

        client
            .execute_ping()
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol")
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_execute_ping_not_enabled() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(false)).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {unexpected:?}"),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_execute_ping() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(true), &tmp_socket).await;

        client
            .execute_ping()
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol")
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_execute_ping_not_enabled() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(false), &tmp_socket).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {unexpected:?}"),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn http_execute_resolver() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_resolver(true)).await;

        let req = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "doit".to_string(),
            component: ResolverFunctionComponent {
                data: ComponentView {
                    properties: serde_json::json!({"salt": "n", "peppa": "pig"}),
                    kind: ComponentKind::Standard,
                },
                parents: vec![
                    ComponentView {
                        properties: serde_json::json!({}),
                        kind: ComponentKind::Standard,
                    },
                    ComponentView {
                        properties: serde_json::json!({}),
                        kind: ComponentKind::Standard,
                    },
                ],
            },
            response_type: cyclone_core::ResolverFunctionResponseType::Object,
            code_base64: base64_encode(
                r#"function doit(input) {
                    console.log(`${Object.keys(input).length}`);
                    console.log('my butt');
                    const v = { a: 'b' };
                    return v;
                }"#,
            ),
            before: vec![],
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "2")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => {
                dbg!(req);
                panic!("output stream ended early")
            }
        };
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "my butt")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => {
                dbg!(req);
                panic!("output stream ended early")
            }
        };
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert!(output.message.starts_with("Output:"));
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        // TODO(fnichol): until we've determined how to handle processing the result server side,
        // we're going to see a heartbeat come back when a request is processed
        match progress.next().await {
            Some(Ok(ProgressMessage::Heartbeat)) => assert!(true),
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive heartbeat: err={err:?}"),
            None => panic!("output stream ended early"),
        }
        match progress.next().await {
            None => assert!(true),
            Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
        };
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(!success.unset);
                assert_eq!(success.data, json!({"a": "b"}));
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn uds_execute_resolver() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_resolver(true), &tmp_socket).await;

        let req = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "doit".to_string(),
            component: ResolverFunctionComponent {
                data: ComponentView {
                    properties: serde_json::json!({"salt": "n", "peppa": "pig"}),
                    kind: ComponentKind::Standard,
                },
                parents: vec![
                    ComponentView {
                        properties: serde_json::json!({}),
                        kind: ComponentKind::Standard,
                    },
                    ComponentView {
                        properties: serde_json::json!({}),
                        kind: ComponentKind::Standard,
                    },
                ],
            },
            response_type: cyclone_core::ResolverFunctionResponseType::Object,
            code_base64: base64_encode(
                r#"function doit(input) {
                    console.log(`${Object.keys(input).length}`);
                    console.log('my butt');
                    const v = { a: 'b' };
                    return v;
                }"#,
            ),
            before: vec![],
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "2")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => {
                dbg!(req);
                panic!("output stream ended early")
            }
        };
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "my butt")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => {
                dbg!(req);
                panic!("output stream ended early")
            }
        };
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert!(output.message.starts_with("Output:"));
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(!success.unset);
                assert_eq!(success.data, json!({"a": "b"}));
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    async fn execute_validation<C, Strm>(mut client: C)
    where
        Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
        C: CycloneClient<Strm>,
    {
        let req = ValidationRequest {
            execution_id: "31337".to_string(),
            handler: "".to_string(),
            value: Some(33.into()),
            validation_format: r#"{"type":"number","flags":{"presence":"required"},"rules":[{"name":"integer"},{"name":"min","args":{"limit":33}},{"name":"max","args":{"limit":33}}]}"#.to_string(),
            code_base64: "".to_string(),
            before: vec![],
        };
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req, Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(success.error.is_none());
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn http_execute_validation() {
        let mut builder = Config::builder();
        let client = http_client_for_running_server(builder.enable_validation(true)).await;

        execute_validation(client).await
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn uds_execute_validation() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let client =
            uds_client_for_running_server(builder.enable_validation(true), &tmp_socket).await;

        execute_validation(client).await
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn http_execute_action_run() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_action_run(true)).await;

        let req = ActionRunRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64_encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { status: 'ok' };
                }"#,
            ),
            before: vec![],
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "first");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'first' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            }
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert!(output.message.starts_with("Output:"));
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive Output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn uds_execute_action_run() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_action_run(true), &tmp_socket).await;

        let req = ActionRunRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64_encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { status: 'ok' };
                }"#,
            ),
            before: vec![],
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "first");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'first' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert!(output.message.starts_with("Output:"));
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn http_execute_schema_variant_definition() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(
            builder.enable_schema_variant_definition(true),
            &tmp_socket,
        )
        .await;

        let req = SchemaVariantDefinitionRequest {
            execution_id: "1234".to_string(),
            handler: "createAsset".to_string(),
            code_base64: base64_encode(
                r#"function createAsset() {
                    console.log('first');
                    console.log('second');
                    return new AssetBuilder().build();
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "first");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'first' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn uds_execute_schema_variant_definition() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(
            builder.enable_schema_variant_definition(true),
            &tmp_socket,
        )
        .await;

        let req = SchemaVariantDefinitionRequest {
            execution_id: "1234".to_string(),
            handler: "createAsset".to_string(),
            code_base64: base64_encode(
                r#"function createAsset() {
                    console.log('first');
                    console.log('second');
                    return new AssetBuilder().build();
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "first");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'first' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn http_execute_management_func() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_management(true), &tmp_socket).await;

        let req = ManagementRequest {
            execution_id: "1234".to_string(),
            handler: "manage".to_string(),
            current_view: "DEFAULT".to_string(),
            this_component: ComponentViewWithGeometry {
                kind: None,
                properties: serde_json::json!({"it": "is", "a": "principle", "of": "music", "to": "repeat the theme"}),
                sources: serde_json::json!({}),
                geometry: serde_json::json!({"x": "1", "y": "2"}),
                incoming_connections: serde_json::json!({}),
            },
            components: HashMap::new(),
            variant_socket_map: HashMap::new(),
            code_base64: base64_encode(
                r#"function manage(input) {
                    console.log('first');
                    console.log('second');
                    return {
                        status: 'ok',
                        message: input.thisComponent.properties.to,
                    }
                }"#,
            ),
            before: vec![],
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "first");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'first' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert_eq!(Some("repeat the theme"), success.message.as_deref());
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test(flavor = "multi_thread", worker_threads = 1))]
    async fn uds_execute_management_func() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_management(true), &tmp_socket).await;

        let req = ManagementRequest {
            execution_id: "1234".to_string(),
            handler: "manage".to_string(),
            current_view: "DEFAULT".to_string(),
            this_component: ComponentViewWithGeometry {
                kind: None,
                properties: serde_json::json!({"it": "is", "a": "principle", "of": "music", "to": "repeat the theme"}),
                sources: serde_json::json!({}),
                geometry: serde_json::json!({"x": "1", "y": "2"}),
                incoming_connections: serde_json::json!({}),
            },
            components: HashMap::new(),
            variant_socket_map: HashMap::new(),
            code_base64: base64_encode(
                r#"function manage({ thisComponent }) {
                    console.log('first');
                    console.log('second');
                    return {
                        status: 'ok',
                        message: thisComponent.properties.to,
                    }
                }"#,
            ),
            before: vec![],
        };

        // Start the protocol
        let mut progress = client
            .prepare_execution(CycloneRequest::from_parts(req.clone(), Default::default()))
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "first");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'first' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => {
                    dbg!(req);
                    panic!("output stream ended early")
                }
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert_eq!(Some("repeat the theme"), success.message.as_deref());
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }
}
