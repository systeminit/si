use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use sdf_core::api_error::ApiError;
use thiserror::Error;

use crate::app_state::AppState;

mod install_workspace;
mod list_workspace_users;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceAPIError {
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] module_index_client::ModuleIndexClientError),
    #[error("module index url not set")]
    ModuleIndexUrlNotSet,
    #[error("cannot export workspace using root tenancy")]
    RootTenancyExportAttempt,
    #[error("cannot install workspace using root tenancy")]
    RootTenancyInstallAttempt,
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("unable to parse url: {0}")]
    Url(#[from] url::ParseError),
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
}

pub type WorkspaceAPIResult<T> = Result<T, WorkspaceAPIError>;

impl IntoResponse for WorkspaceAPIError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            Self::Workspace(dal::WorkspaceError::WorkspaceNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/install", post(install_workspace::install_workspace))
        .route("/users", get(list_workspace_users::list_workspace_users))
}
