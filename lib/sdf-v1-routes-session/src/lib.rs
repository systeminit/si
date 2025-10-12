use axum::{
    Router,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use dal::{
    KeyPairError,
    TransactionsError,
    WorkspaceError,
    WorkspacePk,
    workspace_integrations::WorkspaceIntegrationsError,
};
use hyper::StatusCode;
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_spicedb::SpiceDbError;
use si_events::ulid;
use thiserror::Error;

pub mod auth_connect;
mod refresh_workspace_members;
pub mod restore_authentication;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SessionError {
    #[error("auth api error: {0}")]
    AuthApiError(String),
    #[error("context transactions error: {0}")]
    ContextTransactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    Decode(#[from] ulid::DecodeError),
    #[error("json serialize failed")]
    JSONSerialize(#[from] serde_json::Error),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("login failed")]
    LoginFailed,
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
    #[error("Permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("SpiceDb error: {0}")]
    SpiceDb(#[from] SpiceDbError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace integration error: {0}")]
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
        .route("/connect", post(auth_connect::auth_connect)) // MIGHT BE USED IN NEWHOTNESS
        .route("/reconnect", get(auth_connect::auth_reconnect)) // MIGHT BE USED IN NEWHOTNESS
        .route(
            "/restore_authentication", // MIGHT BE USED IN NEWHOTNESS
            get(restore_authentication::restore_authentication),
        )
        .route(
            "/refresh_workspace_members", // MIGHT BE USED IN NEWHOTNESS (THOUGH MORE DOUBTFUL)
            post(refresh_workspace_members::refresh_workspace_members),
        )
}
