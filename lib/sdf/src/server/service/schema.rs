use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use dal::{SchemaError as DalSchemaError, StandardModelError};
use std::convert::Infallible;
use thiserror::Error;

pub mod create_schema;

#[derive(Debug, Error)]
pub enum SchemaError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("schema error: {0}")]
    Schema(#[from] DalSchemaError),
}

pub type SchemaResult<T> = std::result::Result<T, SchemaError>;

impl IntoResponse for SchemaError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route("/create_schema", post(create_schema::create_schema))
}
