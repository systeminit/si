use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    KeyPairError, StandardModelError, TransactionsError, UserError, UserPk, WorkspaceError,
    WorkspacePk,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::state::AppState;

pub mod auth_connect;
pub mod load_workspaces;
mod refresh_workspace_members;
pub mod restore_authentication;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("auth api error: {0}")]
    AuthApiError(String),
    #[error(transparent)]
    ContextTransactions(#[from] TransactionsError),
    #[error("Invalid user: {0}")]
    InvalidUser(UserPk),
    #[error("Invalid workspace: {0}")]
    InvalidWorkspace(WorkspacePk),
    #[error("json serialize failed")]
    JSONSerialize(#[from] serde_json::Error),
    #[error(transparent)]
    KeyPair(#[from] KeyPairError),
    #[error("login failed")]
    LoginFailed,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthApiErrBody {
    pub kind: String,
    pub message: String,
}

pub type SessionResult<T> = std::result::Result<T, SessionError>;

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message) = match self {
            SessionError::LoginFailed => (StatusCode::CONFLICT, None, None),
            SessionError::InvalidWorkspace(_) => (
                StatusCode::CONFLICT,
                Some("WORKSPACE_NOT_INITIALIZED"),
                None,
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None, None),
        };

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message.unwrap_or(self.to_string()),
                "code": error_code.unwrap_or("42"),
                "statusCode": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/connect", post(auth_connect::auth_connect))
        .route("/reconnect", get(auth_connect::auth_reconnect))
        .route(
            "/restore_authentication",
            get(restore_authentication::restore_authentication),
        )
        .route("/load_workspaces", get(load_workspaces::load_workspaces))
        .route(
            "/refresh_workspace_members",
            post(refresh_workspace_members::refresh_workspace_members),
        )
}
