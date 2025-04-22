use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use dal::{
    KeyPairError, StandardModelError, TransactionsError, UserError, WorkspaceError, WorkspacePk,
    workspace_integrations::WorkspaceIntegrationsError,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use si_data_spicedb::SpiceDbError;
use thiserror::Error;

use sdf_core::{api_error::ApiError, app_state::AppState};

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
    #[error("json serialize failed")]
    JSONSerialize(#[from] serde_json::Error),
    #[error(transparent)]
    KeyPair(#[from] KeyPairError),
    #[error("login failed")]
    LoginFailed,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error("Permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("SpiceDb error: {0}")]
    SpiceDb(#[from] SpiceDbError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error(transparent)]
    WorkspaceIntegration(#[from] WorkspaceIntegrationsError),
    #[error("workspace {0} not yet migrated to new snapshot graph version. Migration required")]
    WorkspaceNotYetMigrated(WorkspacePk),
    #[error("invalid workspace permission: {0}")]
    WorkspacePermission(&'static str),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthApiErrBody {
    pub kind: String,
    pub message: String,
}

pub type SessionResult<T> = std::result::Result<T, SessionError>;

impl IntoResponse for SessionError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            Self::LoginFailed => (StatusCode::CONFLICT, self.to_string()),
            Self::Workspace(WorkspaceError::WorkspaceNotFound(_)) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            Self::WorkspacePermission(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::AuthApiError(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
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
