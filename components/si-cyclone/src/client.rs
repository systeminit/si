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
        let res = self.get("/liveness").await?;

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
