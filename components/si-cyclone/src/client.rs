use crate::{LivenessStatus, LivenessStatusParseError, ReadinessStatus, ReadinessStatusParseError};
use axum::http::request::Builder;
use hyper::{
    body,
    client::{HttpConnector, ResponseFuture},
    Body, Method, Request, Response, StatusCode,
};
use hyperlocal::{UnixClientExt, UnixConnector};
use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::PathBuf,
    str::{self, FromStr},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("invalid liveness status")]
    InvalidLivenessStatus(#[from] LivenessStatusParseError),
    #[error("invalid readiness status")]
    InvalidReadinessStatus(#[from] ReadinessStatusParseError),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error("failed reading http response body")]
    ReadResponseBody(#[source] hyper::Error),
    #[error("failed to create an http request")]
    Request(#[source] hyper::http::Error),
    #[error("http response failed")]
    Response(#[source] hyper::Error),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
    #[error("unexpected status code: {0}")]
    UnexpectedStatusCoce(StatusCode),
    #[error("failed to decode as a UTF8 string")]
    Utf8Decode(#[from] std::str::Utf8Error),
}

#[derive(Clone, Debug)]
pub struct Client {
    http_client: InnerClient,
}

impl Client {
    pub fn new_uds(socket: impl Into<PathBuf>) -> Self {
        let socket = socket.into();
        let http_client = InnerClient::UDS(UDSClient::new(socket));

        Self { http_client }
    }

    pub fn new_http(socket_addrs: impl ToSocketAddrs) -> Result<Self, ClientError> {
        let socket = socket_addrs
            .to_socket_addrs()
            .map_err(ClientError::SocketAddrResolve)?
            .into_iter()
            .next()
            .ok_or(ClientError::NoSocketAddrResolved)?;
        let http_client = InnerClient::HTTP(HTTPClient::new(socket));

        Ok(Self { http_client })
    }

    pub async fn liveness(&self) -> Result<LivenessStatus, ClientError> {
        let res = self.get("/liveness").await?;

        if res.status() != StatusCode::OK {
            return Err(ClientError::UnexpectedStatusCoce(res.status()));
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
            return Err(ClientError::UnexpectedStatusCoce(res.status()));
        }
        let body = body::to_bytes(res)
            .await
            .map_err(ClientError::ReadResponseBody)?;
        let result = ReadinessStatus::from_str(str::from_utf8(body.as_ref())?)?;

        Ok(result)
    }

    async fn get(&self, path: impl AsRef<str>) -> Result<Response<Body>, ClientError> {
        let req = self
            .new_request(path)
            .method(Method::GET)
            .body(Body::empty())
            .map_err(ClientError::Request)?;
        self.request(req).await.map_err(ClientError::Response)
    }

    fn new_request(&self, path: impl AsRef<str>) -> Builder {
        self.http_client.new_request(path)
    }

    fn request(&self, req: Request<Body>) -> ResponseFuture {
        self.http_client.request(req)
    }
}

#[derive(Clone, Debug)]
enum InnerClient {
    HTTP(HTTPClient),
    UDS(UDSClient),
}

impl InnerClient {
    fn new_request(&self, path: impl AsRef<str>) -> Builder {
        match self {
            InnerClient::HTTP(client) => client.request_builder(path),
            InnerClient::UDS(client) => client.request_builder(path),
        }
    }

    fn request(&self, req: Request<Body>) -> ResponseFuture {
        match self {
            InnerClient::HTTP(client) => client.inner.request(req),
            InnerClient::UDS(client) => client.inner.request(req),
        }
    }
}

#[derive(Clone, Debug)]
struct HTTPClient {
    inner: hyper::Client<HttpConnector, Body>,
    socket: SocketAddr,
}

impl HTTPClient {
    fn new(socket: SocketAddr) -> Self {
        let inner = hyper::Client::new();
        Self { inner, socket }
    }

    fn request_builder(&self, path: impl AsRef<str>) -> Builder {
        Request::builder().uri(format!(
            "http://{}:{}/{}",
            self.socket.ip(),
            self.socket.port(),
            path.as_ref().trim_start_matches('/'),
        ))
    }
}

#[derive(Clone, Debug)]
struct UDSClient {
    inner: hyper::Client<UnixConnector, Body>,
    socket: PathBuf,
}

impl UDSClient {
    fn new(socket: PathBuf) -> Self {
        let inner = hyper::Client::unix();
        Self { inner, socket }
    }

    fn request_builder(&self, path: impl AsRef<str>) -> Builder {
        Request::builder().uri(hyperlocal::Uri::new(
            &self.socket,
            &format!("/{}", path.as_ref().trim_start_matches('/')),
        ))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::{NamedTempFile, TempPath};

    use super::*;
    use crate::{
        server::{Config, ConfigBuilder},
        Server,
    };

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
    ) -> Client {
        let server = uds_server(builder, tmp_socket).await;
        let path = server
            .as_uds()
            .expect("server is not uds server")
            .local_path();
        tokio::spawn(async move { server.run().await });

        Client::new_uds(path)
    }

    async fn http_server(builder: &mut ConfigBuilder) -> Server {
        let config = builder
            .http_socket("127.0.0.1:0")
            .expect("failed to resolve socket addr")
            .build()
            .expect("failed to build config");
        Server::init(config).await.expect("failed to init server")
    }

    async fn http_client_for_running_server(builder: &mut ConfigBuilder) -> Client {
        let server = http_server(builder).await;
        let socket = server
            .as_http()
            .expect("server is not an http server")
            .local_addr();
        tokio::spawn(async move { server.run().await });

        Client::new_http(socket).expect("failed to create client")
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
}
