use axum::{
    http::StatusCode, response::IntoResponse, response::Response, routing::get, Json, Router,
};
use dal::{TransactionsError, WsEventError};
use si_data_pg::{PgError, PgPoolError};
use thiserror::Error;

use crate::server::state::AppState;
use crdt::CrdtError;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WsError {
    #[error(transparent)]
    Crdt(#[from] CrdtError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::Error),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
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
