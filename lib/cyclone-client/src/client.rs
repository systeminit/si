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
    ActionRunRequest, ActionRunResultSuccess, LivenessStatus, LivenessStatusParseError,
    ReadinessStatus, ReadinessStatusParseError, ReconciliationRequest, ReconciliationResultSuccess,
    ResolverFunctionRequest, ResolverFunctionResultSuccess, SchemaVariantDefinitionRequest,
    SchemaVariantDefinitionResultSuccess, ValidationRequest, ValidationResultSuccess,
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

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("cannot create client uri")]
    ClientUri(#[source] http::Error),
    #[error("failed to connect")]
    Connect(#[source] Box<dyn std::error::Error + Send + Sync>),
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

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> result::Result<
        Execution<Strm, ResolverFunctionRequest, ResolverFunctionResultSuccess>,
        ClientError,
    >;

    async fn execute_action_run(
        &mut self,
        request: ActionRunRequest,
    ) -> result::Result<Execution<Strm, ActionRunRequest, ActionRunResultSuccess>, ClientError>;

    async fn execute_reconciliation(
        &mut self,
        request: ReconciliationRequest,
    ) -> result::Result<
        Execution<Strm, ReconciliationRequest, ReconciliationResultSuccess>,
        ClientError,
    >;

    async fn execute_validation(
        &mut self,
        request: ValidationRequest,
    ) -> result::Result<Execution<Strm, ValidationRequest, ValidationResultSuccess>, ClientError>;

    async fn execute_schema_variant_definition(
        &mut self,
        request: SchemaVariantDefinitionRequest,
    ) -> result::Result<
        Execution<Strm, SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess>,
        ClientError,
    >;
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

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> Result<Execution<Strm, ResolverFunctionRequest, ResolverFunctionResultSuccess>> {
        let stream = self.websocket_stream("/execute/resolver").await?;
        Ok(execution::execute(stream, request))
    }

    async fn execute_action_run(
        &mut self,
        request: ActionRunRequest,
    ) -> result::Result<Execution<Strm, ActionRunRequest, ActionRunResultSuccess>, ClientError>
    {
        let stream = self.websocket_stream("/execute/command").await?;
        Ok(execution::execute(stream, request))
    }

    async fn execute_reconciliation(
        &mut self,
        request: ReconciliationRequest,
    ) -> result::Result<
        Execution<Strm, ReconciliationRequest, ReconciliationResultSuccess>,
        ClientError,
    > {
        let stream = self.websocket_stream("/execute/reconciliation").await?;
        Ok(execution::execute(stream, request))
    }

    async fn execute_validation(
        &mut self,
        request: ValidationRequest,
    ) -> result::Result<Execution<Strm, ValidationRequest, ValidationResultSuccess>, ClientError>
    {
        Ok(execution::execute(
            self.websocket_stream("/execute/validation").await?,
            request,
        ))
    }

    async fn execute_schema_variant_definition(
        &mut self,
        request: SchemaVariantDefinitionRequest,
    ) -> result::Result<
        Execution<Strm, SchemaVariantDefinitionRequest, SchemaVariantDefinitionResultSuccess>,
        ClientError,
    > {
        Ok(execution::execute(
            self.websocket_stream("/execute/schema_variant_definition")
                .await?,
            request,
        ))
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
    use std::{env, path::Path};

    use base64::{engine::general_purpose, Engine};
    use buck2_resources::Buck2Resources;
    use cyclone_core::{
        ComponentKind, ComponentView, FunctionResult, ProgressMessage, ResolverFunctionComponent,
        ValidationRequest,
    };
    use cyclone_server::{Config, ConfigBuilder, DecryptionKey, Server, UdsIncomingStream};
    use futures::StreamExt;
    use hyper::server::conn::AddrIncoming;
    use serde_json::json;
    use sodiumoxide::crypto::box_::PublicKey;
    use tempfile::{NamedTempFile, TempPath};
    use test_log::test;
    use tracing::warn;

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

    async fn uds_server(
        builder: &mut ConfigBuilder,
        tmp_socket: &TempPath,
        key: DecryptionKey,
    ) -> Server<UdsIncomingStream, PathBuf> {
        let config = builder
            .unix_domain_socket(tmp_socket)
            .try_lang_server_path(lang_server_path())
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
            .try_lang_server_path(lang_server_path())
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

    fn base64_encode(input: impl AsRef<[u8]>) -> String {
        general_purpose::STANDARD_NO_PAD.encode(input)
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
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
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder, key).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_liveness() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket, key).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_readiness() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder, key).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_readiness() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket, key).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
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

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_execute_ping_not_enabled() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(false), key).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {unexpected:?}"),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
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

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_execute_ping_not_enabled() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(false), &tmp_socket, key).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {unexpected:?}"),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
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
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => panic!("output stream ended early"),
        };
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "my butt")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => panic!("output stream ended early"),
        };
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert!(output.message.starts_with("Output:"));
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => panic!("output stream ended early"),
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
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => panic!("output stream ended early"),
        };
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "my butt")
            }
            Some(Ok(unexpected)) => panic!("unexpected msg kind: {unexpected:?}"),
            Some(Err(err)) => panic!("failed to receive 'i like' output: err={err:?}"),
            None => panic!("output stream ended early"),
        };
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert!(output.message.starts_with("Output:"));
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
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
            execution_id: "1337".to_string(),
            handler: "validate".to_string(),
            value: "a string is a sequence of bytes".into(),
            code_base64: base64_encode(
                r"function validate(value) {
                    console.log('i came here to chew bubblegum and validate prop values');
                    console.log('and i\'m all out of gum');
                    if (value === 'a string is a sequence of bytes') {
                        return { valid: true };
                    } else {
                        return { valid: false, message: value + ' is not what i expected' };
                    }
                }",
            ),
        };
        let mut progress = client
            .execute_validation(req)
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(
                        output.message,
                        "i came here to chew bubblegum and validate prop values"
                    );
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => panic!("failed to receive 'bubblegum' output: err={err:?}"),
                None => panic!("output stream ended early"),
            };
        }
        loop {
            match progress.next().await {
                Some(Ok(ProgressMessage::OutputStream(output))) => {
                    assert_eq!(output.message, "and i'm all out of gum");
                    break;
                }
                Some(Ok(ProgressMessage::Heartbeat)) => continue,
                Some(Err(err)) => {
                    panic!("failed to receive 'all out of gum' output: err={err:?}")
                }
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
                Some(unexpected) => panic!("output stream should be done: {unexpected:?}"),
            };
        }
        let result = progress.finish().await.expect("failed to return result");
        match result {
            FunctionResult::Success(success) => {
                assert!(success.valid);
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={failure:?}")
            }
        }
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_execute_validation() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let client = http_client_for_running_server(builder.enable_validation(true), key).await;

        execute_validation(client).await
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn uds_execute_validation() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let client =
            uds_client_for_running_server(builder.enable_validation(true), &tmp_socket, key).await;

        execute_validation(client).await
    }

    #[allow(clippy::disallowed_methods)] // `$RUST_LOG` is checked for in macro
    #[test(tokio::test)]
    async fn http_execute_action_run() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_action_run(true), key).await;

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
        };

        // Start the protocol
        let mut progress = client
            .execute_action_run(req)
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
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => panic!("output stream ended early"),
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
    #[test(tokio::test)]
    async fn uds_execute_action_run() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_action_run(true), &tmp_socket, key).await;

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
        };

        // Start the protocol
        let mut progress = client
            .execute_action_run(req)
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
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
                None => panic!("output stream ended early"),
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
    #[test(tokio::test)]
    async fn http_execute_reconciliation() {
        let (_, key) = gen_keys();
        let mut builder = Config::builder();
        let mut client =
            http_client_for_running_server(builder.enable_reconciliation(true), key).await;

        let req = ReconciliationRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64_encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { updates: { "myid": true }, actions: ["run"] };
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_reconciliation(req)
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
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
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
    #[test(tokio::test)]
    async fn uds_execute_reconciliation() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_reconciliation(true), &tmp_socket, key)
                .await;

        let req = ReconciliationRequest {
            execution_id: "1234".to_string(),
            handler: "workit".to_string(),
            args: Default::default(),
            code_base64: base64_encode(
                r#"function workit() {
                    console.log('first');
                    console.log('second');
                    return { updates: { "myid": true }, actions: ["run"] };
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_reconciliation(req)
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
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
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
    #[test(tokio::test)]
    async fn http_execute_schema_variant_definition() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(
            builder.enable_schema_variant_definition(true),
            &tmp_socket,
            key,
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
            .execute_schema_variant_definition(req)
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
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
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
    #[test(tokio::test)]
    async fn uds_execute_schema_variant_definition() {
        let (_, key) = gen_keys();
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(
            builder.enable_schema_variant_definition(true),
            &tmp_socket,
            key,
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
            .execute_schema_variant_definition(req)
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
                Some(Err(err)) => panic!("failed to receive 'second' output: err={err:?}"),
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
}
