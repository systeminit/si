use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    ActionBatchError, ActionError, ActionPrototypeError, ActionRunnerError,
    ChangeSetError as DalChangeSetPointerError, ChangeSetId, ComponentError, FuncError,
    StandardModelError, TransactionsError, UserError, UserPk, WorkspaceError, WorkspacePk,
    WsEventError,
};
use module_index_client::IndexClientError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::server::state::AppState;

pub mod abandon_change_set;
// mod abandon_vote;
pub mod add_action;
pub mod apply_change_set;
// mod begin_abandon_approval_process;
// mod begin_approval_process;
pub mod create_change_set;
pub mod list_open_change_sets;
pub mod list_queued_actions;
// mod merge_vote;
pub mod remove_action;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error(transparent)]
    Action(#[from] ActionError),
    #[error(transparent)]
    ActionBatch(#[from] ActionBatchError),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    ActionRunner(#[from] ActionRunnerError),
    // #[error("action {0} not found")]
    // ActionNotFound(ActionId),
    #[error("base change set not found for change set: {0}")]
    BaseChangeSetNotFound(ChangeSetId),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    // #[error(transparent)]
    // ChangeStatusError(#[from] ChangeStatusError),
    // #[error(transparent)]
    // Component(#[from] DalComponentError),
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] DalChangeSetPointerError),
    #[error("could not find default change set: {0}")]
    DefaultChangeSetNotFound(ChangeSetId),
    #[error("default change set {0} has no workspace snapshot pointer")]
    DefaultChangeSetNoWorkspaceSnapshotPointer(ChangeSetId),
    #[error(transparent)]
    Func(#[from] FuncError),
    // #[error(transparent)]
    // DalPkg(#[from] dal::pkg::PkgError),
    // #[error(transparent)]
    // Fix(#[from] FixError),
    #[error("invalid header name {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error(transparent)]
    IndexClient(#[from] IndexClientError),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no tenancy set in context")]
    NoTenancySet,
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    // #[error(transparent)]
    // PkgService(#[from] PkgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ChangeSetResult<T> = std::result::Result<T, ChangeSetError>;

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ChangeSetError::ChangeSetNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/list_open_change_sets",
            get(list_open_change_sets::list_open_change_sets),
        )
        .route(
            "/list_queued_actions",
            get(list_queued_actions::list_queued_actions),
        )
        .route("/remove_action", post(remove_action::remove_action))
        .route("/add_action", post(add_action::add_action))
        .route(
            "/create_change_set",
            post(create_change_set::create_change_set),
        )
        .route(
            "/apply_change_set",
            post(apply_change_set::apply_change_set),
        )
        .route(
            "/abandon_change_set",
            post(abandon_change_set::abandon_change_set),
        )
    // .route(
    //     "/begin_approval_process",
    //     post(begin_approval_process::begin_approval_process),
    // )
    // .route(
    //     "/cancel_approval_process",
    //     post(begin_approval_process::cancel_approval_process),
    // )
    // .route("/merge_vote", post(merge_vote::merge_vote))
    // .route(
    //     "/begin_abandon_approval_process",
    //     post(begin_abandon_approval_process::begin_abandon_approval_process),
    // )
    // .route(
    //     "/cancel_abandon_approval_process",
    //     post(begin_abandon_approval_process::cancel_abandon_approval_process),
    // )
    // .route("/abandon_vote", post(abandon_vote::abandon_vote))
}
