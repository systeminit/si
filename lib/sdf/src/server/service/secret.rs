use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Json;
use axum::Router;
use dal::{KeyPairError, StandardModelError};
use std::convert::Infallible;
use thiserror::Error;

pub mod get_public_key;
pub mod list_secrets;

#[derive(Debug, Error)]
pub enum SecretError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    KeyPairError(#[from] KeyPairError),
}

pub type SecretResult<T> = std::result::Result<T, SecretError>;

impl IntoResponse for SecretError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            //SecretError::SecretNotFound => (StatusCode::NOT_FOUND, self.to_string()),
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
        .route("/get_public_key", get(get_public_key::get_public_key))
        .route("/list_secrets", get(list_secrets::list_secrets))
}
