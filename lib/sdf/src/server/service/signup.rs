use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use dal::BillingAccountError;
use std::convert::Infallible;
use thiserror::Error;

pub mod create_account;

#[derive(Debug, Error)]
pub enum SignupError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
}

pub type SignupResult<T> = std::result::Result<T, SignupError>;

impl IntoResponse for SignupError {
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
    Router::new().route("/create_account", post(create_account::create_account))
}
