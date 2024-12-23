use crate::{app_state::AppState, service::ApiError};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use dal::{TransactionsError, UserError, UserPk, WorkspaceError, WorkspacePk};
use thiserror::Error;

mod export_workspace;
mod install_workspace;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceAPIError {
    #[error("Trying to export from/import into root tenancy")]
    ExportingImportingWithRootTenancy,
    #[error("invalid user: {0}")]
    InvalidUser(UserPk),
    #[error("Module index: {0}")]
    ModuleIndex(#[from] module_index_client::ModuleIndexClientError),
    #[error("Module index not configured")]
    ModuleIndexNotConfigured,
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("Unable to parse URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("Could not find current workspace {0}")]
    WorkspaceNotFound(WorkspacePk),
}

pub type WorkspaceAPIResult<T> = Result<T, WorkspaceAPIError>;

impl IntoResponse for WorkspaceAPIError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            WorkspaceAPIError::WorkspaceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/install", post(install_workspace::install_workspace))
        .route("/export", post(export_workspace::export_workspace))
}
