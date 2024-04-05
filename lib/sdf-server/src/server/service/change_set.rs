use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    ActionPrototypeError, ChangeSetApplyError as DalChangeSetApplyError,
    ChangeSetError as DalChangeSetError, ComponentError, DeprecatedActionError, FuncError,
    StandardModelError, TransactionsError, WsEventError,
};

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
    #[error("action error: {0}")]
    Action(#[from] DeprecatedActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] DalChangeSetError),
    #[error("dal change set apply error: {0}")]
    DalChangeSetApply(#[from] DalChangeSetApplyError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("invalid header name {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ws event error: {0}")]
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
