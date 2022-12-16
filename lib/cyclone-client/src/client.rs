use std::{
    marker::PhantomData,
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    result,
    str::{self, FromStr},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use cyclone_core::{
    CommandRunRequest, CommandRunResultSuccess, ConfirmationRequest, ConfirmationResultSuccess,
    LivenessStatus, LivenessStatusParseError, ReadinessStatus, ReadinessStatusParseError,
    ResolverFunctionRequest, ResolverFunctionResultSuccess, WorkflowResolveRequest,
    WorkflowResolveResultSuccess,
};
use http::{
    request::Builder,
    uri::{Authority, InvalidUri, InvalidUriParts, PathAndQuery, Scheme},
};
use hyper::{
    body,
    client::{connect::Connection, HttpConnector, ResponseFuture},
    service::Service,
    Body, Method, Request, Response, StatusCode, Uri,
};
use hyperlocal::{UnixClientExt, UnixConnector, UnixStream};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_tungstenite::WebSocketStream;

use crate::{execution, ping, watch, Execution, PingExecution, Watch};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("cannot create client uri")]
    ClientUri(#[source] http::Error),
    #[error("client is not healthy")]
    Unhealthy(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("invalid liveness status")]
    InvalidLivenessStatus(#[from] LivenessStatusParseError),
    #[error("invalid readiness status")]
    InvalidReadinessStatus(#[from] ReadinessStatusParseError),
    #[error("invalid URI")]
    InvalidUri(#[from] InvalidUri),
    #[error("invalid websocket uri scheme: {0}")]
    InvalidWebsocketScheme(String),
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
    #[error("unexpected status code: {0}")]
    UnexpectedStatusCode(StatusCode),
    #[error("failed to decode as a UTF8 string")]
    Utf8Decode(#[from] std::str::Utf8Error),
    #[error("failed to connect")]
    Connect(#[source] Box<dyn std::error::Error + Send + Sync>),
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

    async fn execute_confirmation(
        &mut self,
        request: ConfirmationRequest,
    ) -> result::Result<Execution<Strm, ConfirmationRequest, ConfirmationResultSuccess>, ClientError>;

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> result::Result<
        Execution<Strm, ResolverFunctionRequest, ResolverFunctionResultSuccess>,
        ClientError,
    >;

    async fn execute_workflow_resolve(
        &mut self,
        request: WorkflowResolveRequest,
    ) -> result::Result<
        Execution<Strm, WorkflowResolveRequest, WorkflowResolveResultSuccess>,
        ClientError,
    >;

    async fn execute_command_run(
        &mut self,
        request: CommandRunRequest,
    ) -> result::Result<Execution<Strm, CommandRunRequest, CommandRunResultSuccess>, ClientError>;
}

impl Client<(), (), ()> {
    pub fn http(
        socket_addrs: impl ToSocketAddrs,
    ) -> Result<Client<HttpConnector, TcpStream, SocketAddr>> {
        let socket = socket_addrs
            .to_socket_addrs()
            .map_err(ClientError::SocketAddrResolve)?
            .into_iter()
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

    pub fn uds(socket: impl Into<PathBuf>) -> Result<Client<UnixConnector, UnixStream, PathBuf>> {
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
}

#[async_trait]
impl<Conn, Strm, Sock> CycloneClient<Strm> for Client<Conn, Strm, Sock>
where
    Conn: Service<Uri, Response = Strm> + Clone + Send + Sync + 'static,
    Conn::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Conn::Future: Unpin + Send,
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + Sync + 'static,
    Sock: Send + Sync,
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

    async fn execute_confirmation(
        &mut self,
        request: ConfirmationRequest,
    ) -> result::Result<Execution<Strm, ConfirmationRequest, ConfirmationResultSuccess>, ClientError>
    {
        let stream = self.websocket_stream("/execute/confirmation").await?;
        Ok(execution::execute(stream, request))
    }

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> Result<Execution<Strm, ResolverFunctionRequest, ResolverFunctionResultSuccess>> {
        let stream = self.websocket_stream("/execute/resolver").await?;
        Ok(execution::execute(stream, request))
    }

    async fn execute_workflow_resolve(
        &mut self,
        request: WorkflowResolveRequest,
    ) -> result::Result<
        Execution<Strm, WorkflowResolveRequest, WorkflowResolveResultSuccess>,
        ClientError,
    > {
        let stream = self.websocket_stream("/execute/workflow").await?;
        Ok(execution::execute(stream, request))
    }

    async fn execute_command_run(
        &mut self,
        request: CommandRunRequest,
    ) -> result::Result<Execution<Strm, CommandRunRequest, CommandRunResultSuccess>, ClientError>
    {
        let stream = self.websocket_stream("/execute/command").await?;
        Ok(execution::execute(stream, request))
    }
}

impl<Conn, Strm, Sock> Client<Conn, Strm, Sock>
where
    Conn: Service<Uri, Response = Strm> + Clone + Send + Sync + 'static,
    Conn::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    Conn::Future: Unpin + Send,
    Strm: AsyncRead + AsyncWrite + Connection + Unpin + Send + Sync + 'static,
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

    fn new_ws_request<P>(&self, path_and_query: P) -> Result<Uri>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let uri = self.ws_request_uri(path_and_query)?;

        // Tokio Tungstenite now requires that the request be perfectly created
        // for websocket upgrades. If you use a URL, everything works.

        //let request = Request::builder()
        //    .uri(uri)
        //    .method(Method::GET)
        //    .body(())
        //    .map_err(ClientError::Request)?;

        Ok(uri)
    }

    async fn get<P>(&self, path_and_query: P) -> Result<Response<Body>>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let request = self
            .new_http_request(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())
            .map_err(ClientError::Request)?;
        self.request(request).await.map_err(ClientError::Response)
    }

    fn request(&self, req: Request<Body>) -> ResponseFuture {
        self.inner_client.request(req)
    }

    async fn websocket_stream<P>(&mut self, path_and_query: P) -> Result<WebSocketStream<Strm>>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let stream = self
            .connector
            .call(self.uri.clone())
            .await
            .map_err(|err| ClientError::Connect(err.into()))?;
        let uri = self.new_ws_request(path_and_query)?;
        let (websocket_stream, response) = tokio_tungstenite::client_async(uri, stream)
            .await
            .map_err(ClientError::WebsocketConnection)?;

        if response.status() != StatusCode::SWITCHING_PROTOCOLS {
            return Err(ClientError::UnexpectedStatusCode(response.status()));
        }

        Ok(websocket_stream)
    }
}

#[derive(Debug)]
struct ClientConfig {
    watch_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            watch_timeout: Duration::from_secs(10),
        }
    }
}

#[allow(clippy::panic, clippy::assertions_on_constants)]
#[cfg(test)]
mod tests {
    use std::{borrow::Cow, env, path::Path};

    use cyclone_core::{
        ComponentKind, ComponentView, FunctionResult, ProgressMessage, ResolverFunctionComponent,
    };
    use cyclone_server::{Config, ConfigBuilder, DecryptionKey, Server, UdsIncomingStream};
    use futures::StreamExt;
    use hyper::server::conn::AddrIncoming;
    use serde_json::json;
    use sodiumoxide::crypto::box_::PublicKey;
    use tempfile::{NamedTempFile, TempPath};
    use test_log::test;

    use super::*;

    fn gen_keys() -> (PublicKey, DecryptionKey) {
        let (pkey, skey) = sodiumoxide::crypto::box_::gen_keypair();
        (pkey, DecryptionKey::from(skey))
    }

    fn rand_uds() -> TempPath {
        NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path()
    }

    fn lang_server_path() -> Cow<'static, str> {
        const ENVVAR: &str = "SI_TEST_LANG_SERVER";
        const DEFAULT: &str = "../../bin/lang-js/target/lang-js";

        env::var(ENVVAR).ok().map(Cow::Owned).unwrap_or_else(|| {
            if !Path::new(DEFAULT).exists() {
                panic!(
                    "lang server not yet built at {}. Override default by setting {}",
                    DEFAULT, ENVVAR
                );
            }
            Cow::Borrowed(DEFAULT)
        })
    }

    async fn uds_server(
        builder: &mut ConfigBuilder,
        tmp_socket: &TempPath,
        key: DecryptionKey,
    ) -> Server<UdsIncomingStream, PathBuf> {
        let config = builder
            .unix_domain_socket(tmp_socket)
            .try_lang_server_path(lang_server_path().to_string())
            .expect("failed to resolve lang server path")
            .build()
            .expect("failed to build config");

        Server::uds(config, Box::new(telemetry::NoopClient), key)
            .await
            .expect("failed to init server")
    }

    async fn uds_client_for_running_server(
        builder: &mut ConfigBuilder,
        tmp_socket: &TempPath,
        key: DecryptionKey,
    ) -> UdsClient {
        let server = uds_server(builder, tmp_socket, key).await;
        let path = server.local_socket().clone();
        tokio::spawn(async move { server.run().await });

        Client::uds(path).expect("failed to create uds client")
    }

    async fn http_server(
        builder: &mut ConfigBuilder,
        key: DecryptionKey,
    ) -> Server<AddrIncoming, SocketAddr> {
        let config = builder
            .http_socket("127.0.0.1:0")
            .expect("failed to resolve socket addr")
            .try_lang_server_path(lang_server_path().to_string())
            .expect("failed to resolve lang server path")
            .build()
            .expect("failed to build config");

        Server::http(config, Box::new(telemetry::NoopClient), key).expect("failed to init server")
    }

    async fn http_client_for_running_server(
        builder: &mut ConfigBuilder,
        key: DecryptionKey,
    ) -> HttpClient {
        let server = http_server(builder, key).await;
        let socket = *server.local_socket();
        tokio::spawn(async move { server.run().await });

        Client::http(socket).expect("failed to create client")
    }

    #[test(tokio::test)]
    async fn http_watch() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client =
            http_client_for_running_server(builder.watch(Some(Duration::from_secs(2))), key).await;

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
                Some(Err(err)) => panic!("failed to receive ping; err={:?}", err),
                None => panic!("stream ended early"),
            }
        }

        // Signal the client's desire to stop the watch
        progress.stop().await.expect("failed to stop protocol");
    }

    #[test(tokio::test)]
    async fn uds_watch() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(
            builder.watch(Some(Duration::from_secs(2))),
            &tmp_socket,
            key,
        )
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
                Some(Err(err)) => panic!("failed to receive ping; err={:?}", err),
                None => panic!("stream ended early"),
            }
        }

        // Signal the client's desire to stop the watch
        progress.stop().await.expect("failed to stop protocol");
    }

    #[test(tokio::test)]
    async fn http_liveness() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder, key).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[test(tokio::test)]
    async fn uds_liveness() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket, key).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[test(tokio::test)]
    async fn http_readiness() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder, key).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[test(tokio::test)]
    async fn uds_readiness() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket, key).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[test(tokio::test)]
    async fn http_execute_ping() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(true), key).await;

        client
            .execute_ping()
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol")
    }

    #[test(tokio::test)]
    async fn http_execute_ping_not_enabled() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(false), key).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {:?}", unexpected),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_ping() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(true), &tmp_socket, key).await;

        client
            .execute_ping()
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol")
    }

    #[test(tokio::test)]
    async fn uds_execute_ping_not_enabled() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(false), &tmp_socket, key).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {:?}", unexpected),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[test(tokio::test)]
    async fn http_execute_resolver() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_resolver(true), key).await;

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
            response_type: cyclone_core::ResolverFunctionResponseType::PropObject,
            code_base64: base64::encode(
                r#"function doit(input) {
                    console.log(`${Object.keys(input).length}`);
                    console.log('my butt');
                    v = { a: 'b' };
                    return v;
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_resolver(req)
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
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {:?}", unexpected),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
            None => panic!("output stream ended early"),
        };
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "my butt")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {:?}", unexpected),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
            None => panic!("output stream ended early"),
        };
        // TODO(fnichol): until we've determined how to handle processing the result server side,
        // we're going to see a heartbeat come back when a request is processed
        match progress.next().await {
            Some(Ok(ProgressMessage::Heartbeat)) => assert!(true),
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {:?}", unexpected),
            Some(Err(err)) => panic!("failed to receive heartbeat: err={:?}", err),
            None => panic!("output stream ended early"),
        }
        match progress.next().await {
            None => assert!(true),
            Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
        };
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(!success.unset);
                assert_eq!(success.data, json!({"a": "b"}));
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_resolver() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_resolver(true), &tmp_socket, key).await;

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
            response_type: cyclone_core::ResolverFunctionResponseType::PropObject,
            code_base64: base64::encode(
                r#"function doit(input) {
                    console.log(`${Object.keys(input).length}`);
                    console.log('my butt');
                    v = { a: 'b' };
                    return v;
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_resolver(req)
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
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {:?}", unexpected),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
            None => panic!("output stream ended early"),
        };
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "my butt")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {:?}", unexpected),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
            None => panic!("output stream ended early"),
        };
        // TODO(fnichol): until we've determined how to handle processing the result server side,
        // we're going to see a heartbeat come back when a request is processed
        match progress.next().await {
            Some(Ok(ProgressMessage::Heartbeat)) => assert!(true),
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {:?}", unexpected),
            Some(Err(err)) => panic!("failed to receive heartbeat: err={:?}", err),
            None => panic!("output stream ended early"),
        }
        match progress.next().await {
            None => assert!(true),
            Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
        };
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(!success.unset);
                assert_eq!(success.data, json!({"a": "b"}));
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn http_execute_confirmation() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client =
            http_client_for_running_server(builder.enable_confirmation(true), key).await;

        let req = ConfirmationRequest {
            execution_id: "1234".to_string(),
            handler: "checkit".to_string(),
            component: ComponentView {
                properties: serde_json::json!({ "si": { "name": "Aipim Frito" } }),
                kind: ComponentKind::Standard,
            },
            code_base64: base64::encode(
                r#"function checkit(component) {
                    console.log('i like');
                    console.log('my butt');
                    if (component.properties.si.name == "Aipim Frito") {
                        return { success: false, recommendedActions: ["fried cassava baby"] };
                    } else {
                        return { success: true, message: "unable to deepfry cassava" };
                    }
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_confirmation(req)
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "i like");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "my butt");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'my butt' output: err={:?}", err),
                None => panic!("output stream ended early"),
            }
        }
        loop {
            match progress.next().await {
                None => {
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
            };
        }
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(!success.success);
                assert_eq!(
                    success.recommended_actions,
                    vec!["fried cassava baby".to_owned()]
                );
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_confirmation() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_confirmation(true), &tmp_socket, key)
                .await;

        let req = ConfirmationRequest {
            execution_id: "1234".to_string(),
            handler: "checkit".to_string(),
            component: ComponentView {
                properties: serde_json::json!({ "si": { "name": "Aipim Frito" } }),
                kind: ComponentKind::Standard,
            },
            code_base64: base64::encode(
                r#"function checkit(component) {
                    console.log('i like');
                    console.log('my butt');
                    if (component.properties.si.name == "Aipim Frito") {
                        return { success: false, recommendedActions: ["fried cassava baby"] };
                    } else {
                        return { success: true, message: "unable to deepfry cassava" };
                    }
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_confirmation(req)
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "i like");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "my butt");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'i like' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                None => {
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(!success.success);
                assert_eq!(
                    success.recommended_actions,
                    vec!["fried cassava baby".to_owned()]
                );
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn http_execute_workflow_resolve() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client =
            http_client_for_running_server(builder.enable_workflow_resolve(true), key).await;

        let req = WorkflowResolveRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64::encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { name: 'name', kind: 'conditional', steps: [] };
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_workflow_resolve(req)
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
                Some(Err(err)) => panic!("failed to receive 'first' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={:?}", err),
                None => panic!("output stream ended early"),
            }
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
            };
        }
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_workflow_resolve() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_workflow_resolve(true), &tmp_socket, key)
                .await;

        let req = WorkflowResolveRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64::encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { name: 'name', kind: 'conditional', steps: [] };
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_workflow_resolve(req)
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
                Some(Err(err)) => panic!("failed to receive 'first' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn http_execute_command_run() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client =
            http_client_for_running_server(builder.enable_command_run(true), key).await;

        let req = CommandRunRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64::encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { status: 'ok' };
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_command_run(req)
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
                Some(Err(err)) => panic!("failed to receive 'first' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={:?}", err),
                None => panic!("output stream ended early"),
            }
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
            };
        }
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_command_run() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_command_run(true), &tmp_socket, key).await;

        let req = CommandRunRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64::encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { status: 'ok' };
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_command_run(req)
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
                Some(Err(err)) => panic!("failed to receive 'first' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "second");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={:?}", err),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                None => {
                    assert!(true);
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(unexpected) => panic!("output stream should be done: {:?}", unexpected),
            };
        }
        // Get the result
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(_success) => {
                // TODO(fnichol): assert some result data
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }
}
