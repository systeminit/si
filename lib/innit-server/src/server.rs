use crate::middleware::client_cert_auth::verify_client_cert_middleware;
use axum::response::{IntoResponse, Response};
use si_data_ssm::ParameterStoreClient;
use si_tls::ClientCertificateVerifier;
use std::io;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

use super::routes;

use axum::{Router, middleware};
use axum::{error_handling::HandleErrorLayer, routing::IntoMakeService};
use hyper::{
    StatusCode,
    server::{accept::Accept, conn::AddrIncoming},
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::{BoxError, ServiceBuilder, buffer::BufferLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use crate::{Config, app_state::AppState};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Signal(#[source] io::Error),
    #[error(transparent)]
    Tls(#[from] si_tls::TlsError),
}

type ServerResult<T> = std::result::Result<T, ServerError>;

pub struct Server<I> {
    inner: axum::Server<I, IntoMakeService<Router>>,
    token: CancellationToken,
}

impl Server<AddrIncoming> {
    pub fn bound_port(&self) -> u16 {
        self.inner.local_addr().port()
    }
}

impl Server<()> {
    #[allow(clippy::too_many_arguments)]
    pub async fn http(
        config: Config,
        token: CancellationToken,
    ) -> ServerResult<Server<AddrIncoming>> {
        let parameter_store_client = ParameterStoreClient::new().await;

        let client_cert_verifier = if let Some(cert) = config.client_ca_cert() {
            Some(Arc::new(
                ClientCertificateVerifier::new(cert.clone()).await?,
            ))
        } else {
            None
        };

        let service = build_service(client_cert_verifier, parameter_store_client, token.clone())?;

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

pub fn build_service(
    client_cert_verifier: Option<Arc<ClientCertificateVerifier>>,
    parameter_store_client: ParameterStoreClient,
    token: CancellationToken,
) -> ServerResult<Router> {
    let state = AppState::new(parameter_store_client, token);

    let routes = routes::routes(state);

    let routes = if let Some(verifier) = client_cert_verifier.clone() {
        routes.layer(middleware::from_fn_with_state(
            verifier,
            verify_client_cert_middleware,
        ))
    } else {
        routes
    };

    let routes = routes
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
