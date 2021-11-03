use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use dal::BillingAccountError;
use std::convert::Infallible;
use thiserror::Error;

mod signup;

#[derive(Debug, Error)]
pub enum TestError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
}

pub type TestResult<T> = std::result::Result<T, TestError>;

impl IntoResponse for TestError {
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
    Router::new().route("/fixtures/signup", post(signup::signup))
}
