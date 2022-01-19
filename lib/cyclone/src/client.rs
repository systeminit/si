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
use http::{
    request::Builder,
    uri::{Authority, InvalidUri, InvalidUriParts, PathAndQuery, Scheme},
};
use hyper::{
    body,
    client::{HttpConnector, ResponseFuture},
    service::Service,
    Body, Method, Request, Response, StatusCode, Uri,
};
use hyperlocal::{UnixClientExt, UnixConnector};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_tungstenite::WebSocketStream;

pub use hyper::client::connect::Connection;
pub use hyperlocal::UnixStream;
pub use tokio_tungstenite::tungstenite::{
    protocol::frame::CloseFrame as WebSocketCloseFrame, Message as WebSocketMessage,
};

pub use self::ping::{PingExecution, PingExecutionError};
pub use self::qualification_check::{
    QualificationCheckExecution, QualificationCheckExecutionClosing,
    QualificationCheckExecutionError, QualificationCheckExecutionStarted,
};
pub use self::resolver_function::{
    ResolverFunctionExecution, ResolverFunctionExecutionClosing, ResolverFunctionExecutionError,
    ResolverFunctionExecutionStarted,
};
pub use self::watch::{Watch, WatchError, WatchStarted};
pub use crate::{
    qualification_check::QualificationCheckRequest, resolver_function::ResolverFunctionRequest,
    LivenessStatus, LivenessStatusParseError, ReadinessStatus, ReadinessStatusParseError,
};

mod ping;
mod qualification_check;
mod resolver_function;
mod watch;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("cannot create client uri")]
    ClientUri(#[source] http::Error),
    #[error("client is not healthy")]
    Unhealty(#[source] Box<dyn std::error::Error + Send + Sync>),
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
        Self::Unhealty(Box::new(source))
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
    ) -> result::Result<ResolverFunctionExecution<Strm>, ClientError>;

    async fn execute_qualification(
        &mut self,
        request: QualificationCheckRequest,
    ) -> result::Result<QualificationCheckExecution<Strm>, ClientError>;
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

    async fn execute_resolver(
        &mut self,
        request: ResolverFunctionRequest,
    ) -> Result<ResolverFunctionExecution<Strm>> {
        let stream = self.websocket_stream("/execute/resolver").await?;
        Ok(resolver_function::execute(stream, request))
    }

    async fn execute_qualification(
        &mut self,
        request: QualificationCheckRequest,
    ) -> Result<QualificationCheckExecution<Strm>> {
        let stream = self.websocket_stream("/execute/qualification").await?;
        Ok(qualification_check::execute(stream, request))
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

    fn new_ws_request<P>(&self, path_and_query: P) -> Result<Request<()>>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let uri = self.ws_request_uri(path_and_query)?;
        let request = Request::builder()
            .uri(uri)
            .method(Method::GET)
            .body(())
            .map_err(ClientError::Request)?;

        Ok(request)
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

#[allow(clippy::panic)]
#[cfg(test)]
mod tests {
    use std::{borrow::Cow, collections::HashMap, env, path::Path};

    use futures::StreamExt;
    use hyper::server::conn::AddrIncoming;
    use serde_json::json;
    use tempfile::{NamedTempFile, TempPath};
    use test_env_log::test;

    use super::*;
    use crate::{
        qualification_check::Component,
        resolver_function::ResolverFunctionRequest,
        server::{Config, ConfigBuilder, UdsIncomingStream},
        FunctionResult, ProgressMessage, Server,
    };

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
    ) -> Server<UdsIncomingStream, PathBuf> {
        let config = builder
            .unix_domain_socket(tmp_socket)
            .lang_server_path(lang_server_path().to_string())
            .build()
            .expect("failed to build config");

        Server::uds(config).await.expect("failed to init server")
    }

    async fn uds_client_for_running_server(
        builder: &mut ConfigBuilder,
        tmp_socket: &TempPath,
    ) -> UdsClient {
        let server = uds_server(builder, tmp_socket).await;
        let path = server.local_socket().clone();
        tokio::spawn(async move { server.run().await });

        Client::uds(path).expect("failed to create uds client")
    }

    async fn http_server(builder: &mut ConfigBuilder) -> Server<AddrIncoming, SocketAddr> {
        let config = builder
            .http_socket("127.0.0.1:0")
            .expect("failed to resolve socket addr")
            .lang_server_path(lang_server_path().to_string())
            .build()
            .expect("failed to build config");

        Server::http(config).expect("failed to init server")
    }

    async fn http_client_for_running_server(builder: &mut ConfigBuilder) -> HttpClient {
        let server = http_server(builder).await;
        let socket = *server.local_socket();
        tokio::spawn(async move { server.run().await });

        Client::http(socket).expect("failed to create client")
    }

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
                Some(Ok(v)) => assert_eq!(v, ()),
                Some(Err(err)) => panic!("failed to receive ping; err={:?}", err),
                None => panic!("stream ended early"),
            }
        }

        // Signal the client's desire to stop the watch
        progress.stop().await.expect("failed to stop protocol");
    }

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
                Some(Ok(v)) => assert_eq!(v, ()),
                Some(Err(err)) => panic!("failed to receive ping; err={:?}", err),
                None => panic!("stream ended early"),
            }
        }

        // Signal the client's desire to stop the watch
        progress.stop().await.expect("failed to stop protocol");
    }

    #[test(tokio::test)]
    async fn http_liveness() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[test(tokio::test)]
    async fn uds_liveness() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[test(tokio::test)]
    async fn http_readiness() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(&mut builder).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[test(tokio::test)]
    async fn uds_readiness() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client = uds_client_for_running_server(&mut builder, &tmp_socket).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

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

    #[test(tokio::test)]
    async fn http_execute_ping_not_enabled() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(false)).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {:?}", unexpected),
            Ok(_) => panic!("stream not expected"),
        }
    }

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

    #[test(tokio::test)]
    async fn uds_execute_ping_not_enabled() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(false), &tmp_socket).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {:?}", unexpected),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[test(tokio::test)]
    async fn http_execute_resolver() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_resolver(true)).await;

        let req = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "doit".to_string(),
            parameters: None,
            code_base64: base64::encode(
                r#"function doit(p) {
                    console.log('i like');
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
                assert_eq!(output.message, "i like")
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
                assert_eq!(success.unset, false);
                assert_eq!(success.data, json!({"a": "b"}));
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_resolver() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_resolver(true), &tmp_socket).await;

        let req = ResolverFunctionRequest {
            execution_id: "1234".to_string(),
            handler: "doit".to_string(),
            parameters: None,
            code_base64: base64::encode(
                r#"function doit(p) {
                    console.log('i like');
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
                assert_eq!(output.message, "i like")
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
                assert_eq!(success.unset, false);
                assert_eq!(success.data, json!({"a": "b"}));
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn http_execute_qualification() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_qualification(true)).await;

        let req = QualificationCheckRequest {
            execution_id: "1234".to_string(),
            handler: "checkit".to_string(),
            component: Component {
                name: "pringles".to_string(),
                properties: HashMap::new(),
            },
            code_base64: base64::encode(
                r#"function checkit(component) {
                    console.log('i like');
                    console.log('my butt');
                    if (component["name"] == "pringles") {
                        return { qualified: true };
                    } else {
                        return {
                            qualified: false,
                            output: "name is not tasty enough",
                        };
                    }
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_qualification(req)
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "i like")
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
                assert_eq!(success.qualified, true);
                assert_eq!(success.output, None);
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }

    #[test(tokio::test)]
    async fn uds_execute_qualification() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_qualification(true), &tmp_socket).await;

        let req = QualificationCheckRequest {
            execution_id: "1234".to_string(),
            handler: "checkit".to_string(),
            component: Component {
                name: "pringles".to_string(),
                properties: HashMap::new(),
            },
            code_base64: base64::encode(
                r#"function checkit(component) {
                    console.log('i like');
                    console.log('my butt');
                    if (component["name"] == "pringles") {
                        return { qualified: true };
                    } else {
                        return {
                            qualified: false,
                            output: "name is not tasty enough",
                        };
                    }
                }"#,
            ),
        };

        // Start the protocol
        let mut progress = client
            .execute_qualification(req)
            .await
            .expect("failed to establish websocket stream")
            .start()
            .await
            .expect("failed to start protocol");

        // Consume the output messages
        match progress.next().await {
            Some(Ok(ProgressMessage::OutputStream(output))) => {
                assert_eq!(output.message, "i like")
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
                assert_eq!(success.qualified, true);
                assert_eq!(success.output, None);
            }
            FunctionResult::Failure(failure) => {
                panic!("result should be success; failure={:?}", failure)
            }
        }
    }
}
