use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    KeyPairError, StandardModelError, TransactionsError, UserError, UserPk, WorkspaceError,
    WorkspacePk,
};
use thiserror::Error;

pub mod auth_connect;
pub mod get_defaults;
pub mod restore_authentication;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    ContextTransactions(#[from] TransactionsError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("login failed")]
    LoginFailed,
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error("json serialize failed")]
    JSONSerialize(#[from] serde_json::Error),
    #[error("auth api error: {0}")]
    AuthApiError(String),
    #[error("Invalid user: {0}")]
    InvalidUser(UserPk),
    #[error("Invalid workspace: {0}")]
    InvalidWorkspace(WorkspacePk),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    KeyPair(#[from] KeyPairError),
}

pub type SessionResult<T> = std::result::Result<T, SessionError>;

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            SessionError::LoginFailed => (StatusCode::CONFLICT, self.to_string()),
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
        .route("/connect", post(auth_connect::auth_connect))
        .route(
            "/restore_authentication",
            get(restore_authentication::restore_authentication),
        )
        .route("/get_defaults", get(get_defaults::get_defaults))
}
