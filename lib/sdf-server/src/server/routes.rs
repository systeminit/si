use crate::server::config::JwtSecretKey;
use axum::{
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use dal::{context::ServicesContext, job::processor::JobQueueProcessor};
use hyper::StatusCode;
use si_data_nats::{NatsClient, NatsError};
use si_data_pg::{PgError, PgPool};
use si_std::SensitiveString;
use tower::ServiceBuilder;
use std::{path::Path, sync::Arc};
use telemetry::TelemetryClient;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};
use veritech_client::{Client as VeritechClient, EncryptionKey};

use super::{
    handlers,
    server::{ServerError, ShutdownSource},
};

pub struct State {
    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    #[allow(dead_code)]
    tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl State {
    pub fn new(tmp_shutdown_tx: mpsc::Sender<ShutdownSource>) -> Self {
        Self { tmp_shutdown_tx }
    }
}

#[derive(Clone, Debug)]
pub struct ShutdownBroadcast(broadcast::Sender<()>);

impl ShutdownBroadcast {
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.0.subscribe()
    }
}

#[derive(Clone, Debug)]
pub struct SignupSecret(Arc<SensitiveString>);

impl SignupSecret {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn routes(
    telemetry: impl TelemetryClient,
    pg_pool: PgPool,
    nats_conn: NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: VeritechClient,
    encryption_key: EncryptionKey,
    jwt_secret_key: JwtSecretKey,
    signup_secret: SensitiveString,
    council_subject_prefix: String,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
    pkgs_path: Option<&Path>,
) -> Router {
    let shared_state = Arc::new(State::new(shutdown_tx));
    let encryption_key = Arc::new(encryption_key);
    let signup_secret = SignupSecret(Arc::new(signup_secret));
    let services_context = ServicesContext::new(
        pg_pool.clone(),
        nats_conn.clone(),
        job_processor,
        veritech.clone(),
        encryption_key.clone(),
        council_subject_prefix,
        pkgs_path.map(|path| path.to_path_buf()),
    );

    let mut router: Router = Router::new();
    router = router
        .route("/api/demo", get(handlers::demo))
        .nest(
            "/api/change_set",
            crate::server::service::change_set::routes(),
        )
        .nest(
            "/api/component",
            crate::server::service::component::routes(),
        )
        .nest("/api/fix", crate::server::service::fix::routes())
        .nest("/api/func", crate::server::service::func::routes())
        .nest("/api/pkg", crate::server::service::pkg::routes())
        .nest("/api/provider", crate::server::service::provider::routes())
        .nest(
            "/api/qualification",
            crate::server::service::qualification::routes(),
        )
        .nest("/api/schema", crate::server::service::schema::routes())
        .nest("/api/diagram", crate::server::service::diagram::routes())
        .nest("/api/secret", crate::server::service::secret::routes())
        .nest("/api/session", crate::server::service::session::routes())
        .nest("/api/signup", crate::server::service::signup::routes())
        .nest("/api/status", crate::server::service::status::routes())
        .nest(
            "/api/variant_def",
            crate::server::service::variant_definition::routes(),
        )
        .nest("/api/workflow", crate::server::service::workflow::routes())
        .nest("/api/ws", crate::server::service::ws::routes());

    // Load test routes if we are in test mode (decided by "opt-level" at the moment).
    router = test_routes(router);

    // Load dev routes if we are in dev mode (decided by "opt-level" at the moment).
    router = dev_routes(router);

    router = router
        .layer(
            ServiceBuilder::new()
                .layer(Extension(shared_state))
                .layer(Extension(services_context))
                .layer(Extension(telemetry))
                .layer(Extension(pg_pool))
                .layer(Extension(nats_conn))
                .layer(Extension(veritech))
                .layer(Extension(encryption_key))
                .layer(Extension(jwt_secret_key))
                .layer(Extension(signup_secret))
                .layer(Extension(ShutdownBroadcast(shutdown_broadcast_tx)))
        );
    router
}

#[cfg(debug_assertions)]
pub fn test_routes(mut router: Router) -> Router {
    router = router.nest("/api/test", crate::server::service::test::routes());
    router
}

#[cfg(not(debug_assertions))]
pub fn test_routes(router: Router) -> Router {
    telemetry::prelude::debug!("skipping test routes...");
    router
}

#[cfg(debug_assertions)]
pub fn dev_routes(mut router: Router) -> Router {
    router = router.nest("/api/dev", crate::server::service::dev::routes());
    router
}

#[cfg(not(debug_assertions))]
pub fn dev_routes(router: Router) -> Router {
    telemetry::prelude::debug!("skipping dev routes...");
    router
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16(),
            },
        }));

        (status, body).into_response()
    }
}
