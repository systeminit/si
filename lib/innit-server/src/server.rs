use aws_sdk_ssm::Client;
use axum::response::{IntoResponse, Response};
use std::io;
use tokio_util::sync::CancellationToken;

use super::routes;

use axum::Router;
use axum::{error_handling::HandleErrorLayer, routing::IntoMakeService};
use hyper::{
    server::{accept::Accept, conn::AddrIncoming},
    StatusCode,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::{buffer::BufferLayer, BoxError, ServiceBuilder};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{app_state::AppState, Config};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
}

type ServerResult<T> = std::result::Result<T, ServerError>;

pub struct Server<I> {
    inner: axum::Server<I, IntoMakeService<Router>>,
    token: CancellationToken,
}

impl Server<()> {
    #[allow(clippy::too_many_arguments)]
    pub async fn http(
        config: Config,
        token: CancellationToken,
    ) -> ServerResult<Server<AddrIncoming>> {
        let aws_config = aws_config::load_from_env().await;
        let ssm_client = Client::new(&aws_config);

        let service = build_service(ssm_client, token.clone())?;

        info!(
            "binding to HTTP socket; socket_addr={}",
            config.socket_addr()
        );
        let inner = axum::Server::bind(config.socket_addr()).serve(service.into_make_service());

        Ok(Server { inner, token })
    }
}

impl<I, IO, IE> Server<I>
where
    I: Accept<Conn = IO, Error = IE>,
    IO: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    IE: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn run(self) -> ServerResult<()> {
        self.inner
            .with_graceful_shutdown(async { self.token.cancelled().await })
            .await
            .map_err(Into::into)
    }
}

pub fn build_service(client: Client, token: CancellationToken) -> ServerResult<Router> {
    let state = AppState::new(client, token);

    let routes = routes::routes(state)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    tracing::error!(error = %err, "Unexpected error in request processing");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Internal server error: {}", err))
                        .expect("Unable to build error response body")
                        .into_response()
                }))
                .layer(BufferLayer::new(128)),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    Ok(routes)
}
