use std::{
    io,
    sync::Arc,
};

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    middleware,
    response::{
        IntoResponse,
        Response,
    },
    routing::IntoMakeService,
};
use hyper::{
    StatusCode,
    server::{
        accept::Accept,
        conn::AddrIncoming,
    },
};
use si_data_acmpca::{
    PrivateCertManagerClient,
    PrivateCertManagerClientError,
};
use si_data_ssm::ParameterStoreClient;
use si_tls::{
    CertificateSource,
    ClientCertificateVerifier,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::io::{
    AsyncRead,
    AsyncWrite,
};
use tokio_util::sync::CancellationToken;
use tower::{
    BoxError,
    ServiceBuilder,
    buffer::BufferLayer,
};
use tower_http::trace::{
    DefaultMakeSpan,
    TraceLayer,
};

use super::routes;
use crate::{
    Config,
    app_state::AppState,
    middleware::client_cert_auth::verify_client_cert_middleware,
    parameter_cache::ParameterCache,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    CertificateClient(#[from] PrivateCertManagerClientError),
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
        let parameter_store_client = if let Some(endpoint) = config.test_endpoint() {
            ParameterStoreClient::new_for_test(endpoint)
        } else {
            ParameterStoreClient::new().await
        };

        // dev mode, no certs necessary
        // use a cert if one is supplied
        // or get one from aws if the arn is supplied
        // otherwise, no certs necessary
        let client_cert_verifier = if config.dev_mode() {
            None
        } else if let Some(certs) = config.client_ca_certs() {
            Some(Arc::new(
                ClientCertificateVerifier::new(&certs.clone()).await?,
            ))
        } else if let Some(ca_arns) = config.client_ca_arns() {
            Some(Arc::new(
                ClientCertificateVerifier::new(&get_ca_certs_from_arns(ca_arns).await?).await?,
            ))
        } else {
            None
        };

        let parameter_cache = ParameterCache::new(config.cache_ttl());

        let service = build_service(
            client_cert_verifier,
            parameter_cache,
            parameter_store_client,
            token.clone(),
        )?;

        info!(
            "binding to HTTP socket; socket_addr={}",
            config.socket_addr()
        );
        let inner = axum::Server::bind(config.socket_addr()).serve(service.into_make_service());

        Ok(Server { inner, token })
    }
}

async fn get_ca_certs_from_arns(ca_cert_arns: &[String]) -> ServerResult<Vec<CertificateSource>> {
    let acmpca_client = PrivateCertManagerClient::new().await;
    let mut ca_certs = vec![];
    for ca_cert_arn in ca_cert_arns {
        info!("Fetching CA Cert for ARN: {ca_cert_arn}");
        ca_certs.push(
            acmpca_client
                .get_certificate_authority(ca_cert_arn.to_string())
                .await?,
        );
    }
    Ok(ca_certs)
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
    parameter_cache: ParameterCache,
    parameter_store_client: ParameterStoreClient,
    token: CancellationToken,
) -> ServerResult<Router> {
    let state = AppState::new(parameter_cache, parameter_store_client, token);

    let routes = routes::routes(state);

    let routes = if let Some(verifier) = client_cert_verifier.clone() {
        info!("Configuring the server for client cert validation");
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
