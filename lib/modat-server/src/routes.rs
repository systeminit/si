use std::sync::Arc;

use axum::{
    response::{IntoResponse, Response},
    Extension, Json, Router,
};
use dal::ServicesContext;
use hyper::StatusCode;
use si_data::{nats, pg};
use telemetry::TelemetryClient;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc};

use crate::server::{ServerError, ShutdownSource};

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

#[must_use]
pub fn routes(
    telemetry: impl TelemetryClient,
    pg_pool: pg::PgPool,
    nats_conn: nats::Client,
    veritech: veritech::Client,
    encryption_key: veritech::EncryptionKey,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
    shutdown_broadcast_tx: broadcast::Sender<()>,
) -> Router {
    let shared_state = Arc::new(State::new(shutdown_tx));
    let encryption_key = Arc::new(encryption_key);
    let services_context = ServicesContext::new(pg_pool, nats_conn, veritech, encryption_key);
    let mut router: Router = Router::new();
    router = router
        .layer(Extension(shared_state))
        .layer(Extension(services_context))
        .layer(Extension(telemetry))
        .layer(Extension(ShutdownBroadcast(shutdown_broadcast_tx)));
    router
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] nats::Error),
    #[error(transparent)]
    Pg(#[from] pg::Error),
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
