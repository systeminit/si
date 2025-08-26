use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};
use dal::ChangeSetId;
use sdf_core::EddaClientError;
use thiserror::Error;

pub mod create;
pub mod delete;
pub mod force_apply;
pub mod get;
pub mod list;
pub mod merge_status;
pub mod purge_open;
pub mod request_approval;

use super::common::ErrorIntoResponse;

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("action error: {0}")]
    Action(#[from] dal::action::ActionError),
    #[error("cannot abandon head change set")]
    CannotAbandonHead,
    #[error("cannot merge head change set")]
    CannotMergeHead,
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("change set not found: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("edda client error: {0}")]
    EddaClient(#[from] EddaClientError),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

impl ErrorIntoResponse for ChangeSetError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            ChangeSetError::ChangeSet(dal::ChangeSetError::ChangeSetNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ChangeSetError::ChangeSet(dal::ChangeSetError::DvuRootsNotEmpty(_)) => (
                StatusCode::PRECONDITION_REQUIRED,
                "There are dependent values that still need to be calculated. Please retry!"
                    .to_string(),
            ),
            ChangeSetError::CannotAbandonHead => (StatusCode::BAD_REQUEST, self.to_string()),
            ChangeSetError::CannotMergeHead => (StatusCode::BAD_REQUEST, self.to_string()),
            ChangeSetError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
        self.to_api_response()
    }
}

impl From<JsonRejection> for ChangeSetError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                ChangeSetError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                ChangeSetError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => ChangeSetError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => ChangeSetError::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}
