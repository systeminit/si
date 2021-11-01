use std::{convert::Infallible, sync::Arc};

use axum::{
    body::{Bytes, Full},
    handler::get,
    response::IntoResponse,
    routing::BoxRoute,
    AddExtensionLayer, Json, Router,
};
use hyper::StatusCode;
use serde_json::json;
use si_data::{nats, pg};
use thiserror::Error;
use tokio::sync::mpsc;

use super::{
    handlers,
    server::{ServerError, ShutdownSource},
};

struct State {
    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    #[allow(dead_code)]
    tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl State {
    fn new(tmp_shutdown_tx: mpsc::Sender<ShutdownSource>) -> Self {
        Self { tmp_shutdown_tx }
    }
}

#[must_use]
pub fn routes(
    pg_pool: pg::PgPool,
    nats: nats::NatsConn,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
) -> Router<BoxRoute> {
    let shared_state = Arc::new(State::new(shutdown_tx));

    Router::new()
        .route("/demo", get(handlers::demo))
        .layer(AddExtensionLayer::new(shared_state))
        .layer(AddExtensionLayer::new(pg_pool))
        .layer(AddExtensionLayer::new(nats))
        .boxed()
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsTxnError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            AppError::Nats(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::Pg(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            // Fallback which takes all server error and returns them as an internal server error.
            // Note that having higher order, semantic responses it much preferred.
            AppError::Server(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}
