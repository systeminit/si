use axum::{
    http::StatusCode, response::IntoResponse, response::Response, routing::get, Json, Router,
};
use crdt::CrdtError;
use dal::{TransactionsError, WsEventError};
use nats_multiplexer_client::MultiplexerClientError;
use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;
use tokio::sync::TryLockError;

use crate::server::state::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WsError {
    #[error("crdt error: {0}")]
    Crdt(#[from] CrdtError),
    #[error("nats multiplexer client error: {0}")]
    MultiplexerClient(#[from] MultiplexerClientError),
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("wsevent error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub mod crdt;
pub mod workspace_updates;

impl IntoResponse for WsError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/workspace_updates",
            get(workspace_updates::workspace_updates),
        )
        .route("/crdt", get(crdt::crdt))
}
