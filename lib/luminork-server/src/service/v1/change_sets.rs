use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

// Import submodules
pub mod create;
pub mod force_apply;
pub mod get;
pub mod list;
pub mod merge_status;
pub mod request_approval;

use super::common::ErrorIntoResponse;

// Common error type for all change set operations
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("action error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

impl ErrorIntoResponse for ChangeSetError {
    fn status_and_message(&self) -> (StatusCode, String) {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
    }
}

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
        self.to_api_response()
    }
}
