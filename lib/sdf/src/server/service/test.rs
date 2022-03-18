use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use dal::{BillingAccountError, TransactionsError, UserError};
use std::convert::Infallible;
use thiserror::Error;

mod signup;
mod signup_and_login;

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum TestError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
    #[error("user error: {0}")]
    User(#[from] UserError),
}

pub type TestResult<T> = std::result::Result<T, TestError>;

impl IntoResponse for TestError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
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

pub fn routes() -> Router {
    Router::new()
        .route("/fixtures/signup", post(signup::signup))
        .route(
            "/fixtures/signup_and_login",
            post(signup_and_login::signup_and_login),
        )
}
