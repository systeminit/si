use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use dal::{BillingAccountError, TransactionsError};
use thiserror::Error;

pub mod create_account;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum SignupError {
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("invalid signup secret")]
    InvalidSignupSecret,
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
}

pub type SignupResult<T> = std::result::Result<T, SignupError>;

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SignupError::InvalidSignupSecret => {
                (StatusCode::BAD_REQUEST, "signup failed".to_string())
            }
            err => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

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
    Router::new().route("/create_account", post(create_account::create_account))
}
