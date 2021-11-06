use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{BillingAccountError, StandardModelError, UserError};
use std::convert::Infallible;
use thiserror::Error;

pub mod login;
pub mod restore_authentication;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsTxnError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("billing account error: {0}")]
    BillingAccount(#[from] BillingAccountError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("login failed")]
    LoginFailed,
}

pub type SessionResult<T> = std::result::Result<T, SessionError>;

impl IntoResponse for SessionError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            SessionError::LoginFailed => (StatusCode::UNAUTHORIZED, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new().route("/login", post(login::login)).route(
        "/restore_authentication",
        get(restore_authentication::restore_authentication),
    )
}
