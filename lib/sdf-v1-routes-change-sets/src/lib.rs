use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::post,
};
use dal::{
    ActionPrototypeId,
    ChangeSetApplyError as DalChangeSetApplyError,
    ChangeSetError as DalChangeSetError,
    ChangeSetId,
    ComponentError,
    FuncError,
    SchemaError,
    SchemaVariantError,
    TransactionsError,
    WorkspaceError,
    WorkspaceSnapshotError,
    WsEventError,
    action::{
        ActionError,
        prototype::ActionPrototypeError,
    },
};
use sdf_core::{
    EddaClientError,
    api_error::ApiError,
    app_state::AppState,
};
use telemetry::prelude::*;
use thiserror::Error;

pub mod abandon_change_set;
pub mod add_action;
pub mod create_change_set;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action already enqueued: {0}")]
    ActionAlreadyEnqueued(ActionPrototypeId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("cannot abandon head change set")]
    CannotAbandonHead,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] DalChangeSetError),
    #[error("dal change set apply error: {0}")]
    DalChangeSetApply(#[from] DalChangeSetApplyError),
    #[error("dvu roots are not empty for change set: {0}")]
    DvuRootsNotEmpty(ChangeSetId),
    #[error("edda client error: {0}")]
    EddaClient(#[from] EddaClientError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("invalid header name {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ChangeSetResult<T> = std::result::Result<T, ChangeSetError>;

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ChangeSetError::ActionAlreadyEnqueued(_) => {
                (StatusCode::NOT_MODIFIED, self.to_string())
            }
            ChangeSetError::DalChangeSet(DalChangeSetError::ChangeSetNotFound(..)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ChangeSetError::DalChangeSetApply(_) => (StatusCode::CONFLICT, self.to_string()),
            ChangeSetError::DvuRootsNotEmpty(_) => (
                StatusCode::PRECONDITION_REQUIRED,
                "There are dependent values that still need to be calculated. Please retry!"
                    .to_string(),
            ),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/add_action", post(add_action::add_action)) // USED IN OLD UI
        .route(
            "/create_change_set", // USED IN NEW UI
            post(create_change_set::create_change_set),
        )
        .route(
            "/abandon_change_set", // USED IN NEW UI
            post(abandon_change_set::abandon_change_set),
        )
}
