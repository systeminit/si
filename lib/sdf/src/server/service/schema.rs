use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    SchemaError as DalSchemaError, SchemaKind, StandardModelError, TransactionsError, WsEventError,
};
use thiserror::Error;

pub mod create_schema;
pub mod get_schema;
pub mod list_schemas;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("no diagram kind for schema kind {0}")]
    NoDiagramKindForSchemaKind(SchemaKind),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("schema error: {0}")]
    Schema(#[from] DalSchemaError),
    #[error("schema not found")]
    SchemaNotFound,
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

pub fn routes() -> Router {
    Router::new()
        .route("/create_schema", post(create_schema::create_schema))
        .route("/list_schemas", get(list_schemas::list_schemas))
        .route("/get_schema", get(get_schema::get_schema))
}
