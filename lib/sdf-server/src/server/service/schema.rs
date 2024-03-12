use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use dal::{SchemaError as DalSchemaError, StandardModelError, TransactionsError, WsEventError};
use thiserror::Error;

use crate::server::state::AppState;

pub mod list_schemas;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error("schema error: {0}")]
    Schema(#[from] DalSchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("wsevent error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaResult<T> = std::result::Result<T, SchemaError>;

impl IntoResponse for SchemaError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SchemaError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/list_schemas", get(list_schemas::list_schemas))
}
