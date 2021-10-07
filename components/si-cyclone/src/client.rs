use crate::{LivenessStatus, LivenessStatusParseError, ReadinessStatus, ReadinessStatusParseError};
use axum::http::request::Builder;
use http::uri::{Authority, InvalidUri, InvalidUriParts, PathAndQuery, Scheme};
use hyper::{
    body,
    client::{connect::Connection, HttpConnector, ResponseFuture},
    service::Service,
    Body, Method, Request, Response, StatusCode, Uri,
};
use hyperlocal::{UnixClientExt, UnixConnector};
use std::{
    convert::{TryFrom, TryInto},
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    str::{self, FromStr},
};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::WebSocketStream;

pub use tokio_tungstenite::tungstenite::Message as WebSocketMessage;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("cannot create client uri")]
    ClientUri(#[source] http::Error),
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

#[derive(Clone, Debug)]
pub struct Client<C, S> {
    inner_client: hyper::Client<C, Body>,
    connector: C,
    socket: S,
    uri: Uri,
}

pub type UDSClient = Client<UnixConnector, PathBuf>;
pub type HTTPClient = Client<HttpConnector, SocketAddr>;

impl Client<(), ()> {
    pub fn http(
        socket_addrs: impl ToSocketAddrs,
    ) -> Result<Client<HttpConnector, SocketAddr>, ClientError> {
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

        Ok(Client {
            inner_client,
            connector,
            socket,
            uri,
        })
    }

    pub fn uds(socket: impl Into<PathBuf>) -> Result<Client<UnixConnector, PathBuf>, ClientError> {
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
            inner_client,
            connector,
            socket,
            uri,
        })
    }
}

impl<C, S, T> Client<C, S>
where
    C: Service<Uri, Response = T> + Clone + Send + Sync + 'static,
    C::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    C::Future: Unpin + Send,
    T: AsyncRead + AsyncWrite + Connection + Unpin + Send + 'static,
{
    pub async fn liveness(&self) -> Result<LivenessStatus, ClientError> {
        let res = self.get("/liveness").await?;

        if res.status() != StatusCode::OK {
            return Err(ClientError::UnexpectedStatusCode(res.status()));
        }
        let body = body::to_bytes(res)
            .await
            .map_err(ClientError::ReadResponseBody)?;
        let result = LivenessStatus::from_str(str::from_utf8(body.as_ref())?)?;

        Ok(result)
    }

    pub async fn readiness(&self) -> Result<ReadinessStatus, ClientError> {
        let res = self.get("/readiness").await?;

        if res.status() != StatusCode::OK {
            return Err(ClientError::UnexpectedStatusCode(res.status()));
        }
        let body = body::to_bytes(res)
            .await
            .map_err(ClientError::ReadResponseBody)?;
        let result = ReadinessStatus::from_str(str::from_utf8(body.as_ref())?)?;

        Ok(result)
    }

    pub async fn execute_ping(&mut self) -> Result<WebSocketStream<T>, ClientError> {
        self.websocket_stream("/execute/ping").await
    }

    fn http_request_uri<P>(&self, path_and_query: P) -> Result<Uri, ClientError>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let mut parts = self.uri.clone().into_parts();
        parts.path_and_query = Some(path_and_query.try_into()?);
        let uri = Uri::from_parts(parts).map_err(ClientError::RequestUri)?;

        Ok(uri)
    }

    fn ws_request_uri<P>(&self, path_and_query: P) -> Result<Uri, ClientError>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let mut parts = self.uri.clone().into_parts();
        let uri_scheme = parts.scheme.take();
        match uri_scheme {
            Some(scheme) => match scheme.as_str() {
                "http" | "unix" => {
                    let _ = parts.scheme.replace(Scheme::try_from("ws")?);
                }
                "https" => {
                    let _ = parts.scheme.replace(Scheme::try_from("wss")?);
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

    fn new_http_request<P>(&self, path_and_query: P) -> Result<Builder, ClientError>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let uri = self.http_request_uri(path_and_query)?;

        Ok(Request::builder().uri(uri))
    }

    fn new_ws_request<P>(&self, path_and_query: P) -> Result<Request<()>, ClientError>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let uri = self.ws_request_uri(path_and_query)?;
        let req = Request::builder()
            .uri(uri)
            .method(Method::GET)
            .body(())
            .map_err(ClientError::Request)?;

        Ok(req)
    }

    async fn get<P>(&self, path_and_query: P) -> Result<Response<Body>, ClientError>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let req = self
            .new_http_request(path_and_query)?
            .method(Method::GET)
            .body(Body::empty())
            .map_err(ClientError::Request)?;
        self.request(req).await.map_err(ClientError::Response)
    }

    fn request(&self, req: Request<Body>) -> ResponseFuture {
        self.inner_client.request(req)
    }

    async fn websocket_stream<P>(
        &mut self,
        path_and_query: P,
    ) -> Result<WebSocketStream<T>, ClientError>
    where
        P: TryInto<PathAndQuery, Error = InvalidUri>,
    {
        let stream = self
            .connector
            .call(self.uri.clone())
            .await
            .map_err(|err| ClientError::Connect(err.into()))?;
        let req = self.new_ws_request(path_and_query)?;
        let (websocket_stream, res) = tokio_tungstenite::client_async(req, stream)
            .await
            .map_err(ClientError::WebsocketConnection)?;

        if res.status() != StatusCode::SWITCHING_PROTOCOLS {
            return Err(ClientError::UnexpectedStatusCode(res.status()));
        }

        Ok(websocket_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        server::{Config, ConfigBuilder},
        Server,
    };
    use futures::StreamExt;
    use tempfile::{NamedTempFile, TempPath};

    fn rand_uds() -> TempPath {
        NamedTempFile::new()
            .expect("failed to create named tempfile")
            .into_temp_path()
    }

    async fn uds_server(builder: &mut ConfigBuilder, tmp_socket: &TempPath) -> Server {
        let config = builder
            .unix_domain_socket(tmp_socket)
            .build()
            .expect("failed to build config");

        Server::init(config).await.expect("failed to init server")
    }

    async fn uds_client_for_running_server(
        builder: &mut ConfigBuilder,
        tmp_socket: &TempPath,
    ) -> UDSClient {
        let server = uds_server(builder, tmp_socket).await;
        let path = server
            .as_uds()
            .expect("server is not uds server")
            .local_path();
        tokio::spawn(async move { server.run().await });

        Client::uds(path).expect("failed to create uds client")
    }

    async fn http_server(builder: &mut ConfigBuilder) -> Server {
        let config = builder
            .http_socket("127.0.0.1:0")
            .expect("failed to resolve socket addr")
            .build()
            .expect("failed to build config");
        Server::init(config).await.expect("failed to init server")
    }

    async fn http_client_for_running_server(builder: &mut ConfigBuilder) -> HTTPClient {
        let server = http_server(builder).await;
        let socket = server
            .as_http()
            .expect("server is not an http server")
            .local_addr();
        tokio::spawn(async move { server.run().await });

        Client::http(socket).expect("failed to create client")
    }

    #[tokio::test]
    async fn http_liveness() {
        let mut builder = Config::builder();
        let client = http_client_for_running_server(&mut builder).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[tokio::test]
    async fn uds_liveness() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let client = uds_client_for_running_server(&mut builder, &tmp_socket).await;

        let response = client.liveness().await.expect("failed to get liveness");

        assert_eq!(response, LivenessStatus::Ok);
    }

    #[tokio::test]
    async fn http_readiness() {
        let mut builder = Config::builder();
        let client = http_client_for_running_server(&mut builder).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[tokio::test]
    async fn uds_readiness() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let client = uds_client_for_running_server(&mut builder, &tmp_socket).await;

        let response = client.readiness().await.expect("failed to get readiness");

        assert_eq!(response, ReadinessStatus::Ready);
    }

    #[tokio::test]
    async fn http_execute_ping() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(true)).await;

        let (_, mut rx) = client
            .execute_ping()
            .await
            .expect("failed to get stream")
            .split();

        match rx.next().await {
            Some(Ok(WebSocketMessage::Text(text))) => assert_eq!("pong", text),
            Some(Ok(unexpected)) => panic!("unexpected message type: {}", unexpected),
            Some(Err(err)) => panic!("unexpected error: {}", err),
            None => panic!("websocket stream should contain a message"),
        }
    }

    #[tokio::test]
    async fn http_execute_ping_not_enabled() {
        let mut builder = Config::builder();
        let mut client = http_client_for_running_server(builder.enable_ping(false)).await;

        match client.execute_ping().await {
            Err(ClientError::WebsocketConnection(_)) => assert!(true),
            Err(unexpected) => panic!("unexpected error: {:?}", unexpected),
            Ok(_) => panic!("stream not expected"),
        }
    }

    #[tokio::test]
    async fn uds_execute_ping() {
        let tmp_socket = rand_uds();
        let mut builder = Config::builder();
        let mut client =
            uds_client_for_running_server(builder.enable_ping(true), &tmp_socket).await;

        let (_, mut rx) = client
            .execute_ping()
            .await
            .expect("failed to get stream")
            .split();

        match rx.next().await {
            Some(Ok(WebSocketMessage::Text(text))) => assert_eq!("pong", text),
            Some(Ok(unexpected)) => panic!("unexpected message type: {}", unexpected),
            Some(Err(err)) => panic!("unexpected error: {:?}", err),
            None => panic!("websocket stream should contain a message"),
        }
    }

    #[tokio::test]
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
}
