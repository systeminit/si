use std::sync::Arc;

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    response::{
        IntoResponse,
        Response,
    },
    routing::IntoMakeService,
};
use bedrock_core::ArtifactStoreConfig;
use hyper::{
    StatusCode,
    server::{
        accept::Accept,
        conn::AddrIncoming,
    },
};
use s3::creds::Credentials;
use serde_json::json;
use si_data_acmpca::PrivateCertManagerClientError;
use si_data_nats::{
    NatsClient,
    NatsConfig,
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
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    trace::{
        DefaultMakeSpan,
        TraceLayer,
    },
};

use super::routes;
use crate::{
    Config,
    ServerResult,
    app_state::AppState,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("AWS credentials error: {0}")]
    AwsCredentials(String),
    #[error(transparent)]
    CertificateClient(#[from] PrivateCertManagerClientError),
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("failed to setup signal handler")]
    Tls(#[from] si_tls::TlsError),
}

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
        let nats = Self::connect_to_nats(config.nats()).await?;

        let session_token = config.aws_session_token();
        let session_token_opt = if session_token.is_empty() {
            None
        } else {
            Some(session_token.as_str())
        };

        let aws_credentials = Credentials::new(
            Some(&config.aws_access_key_id()),
            Some(&config.aws_secret_access_key()),
            session_token_opt,
            None,
            None,
        )
        .map_err(|e| ServerError::AwsCredentials(e.to_string()));

        let service = build_service(Arc::new(nats), aws_credentials.unwrap(), token.clone())?;

        info!(
            "binding to HTTP socket; socket_addr={}",
            config.socket_addr()
        );
        let inner = axum::Server::bind(config.socket_addr()).serve(service.into_make_service());

        Ok(Server { inner, token })
    }

    #[instrument(name = "bedrock.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> ServerResult<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats server");
        Ok(client)
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
    nats: Arc<NatsClient>,
    credentials: Credentials,
    token: CancellationToken,
) -> ServerResult<Router> {
    let artifact_config = ArtifactStoreConfig {
        variant: "s3".to_string(),
        metadata: json!({
            "bucketName": "si-artifacts-prod"
        }),
    };

    let state = AppState::new(nats, artifact_config, credentials, token);

    let public_routes = routes::public_routes(state.clone());

    let routes = public_routes
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let routes = routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    tracing::error!(error = %err, "Unexpected error in request processing");
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(format!("Internal server error: {err}"))
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
