use std::result;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dal::{ChangeSetId, ChangeSetStatus, WsEventError};
use thiserror::Error;

use crate::{middleware::WorkspacePermissionLayer, service::ApiError, AppState};

mod apply;
mod approve;
mod cancel_approval_request;
mod force_apply;
mod list;
mod reject;
mod reopen;
mod request_approval;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("change set not approved for apply. Current state: {0}")]
    ChangeSetNotApprovedForApply(ChangeSetStatus),
    #[error("change set not found: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("dvu roots are not empty for change set: {0}")]
    DvuRootsNotEmpty(ChangeSetId),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("spicedb not found")]
    SpiceDBNotFound,
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("found an unexpected number of open change sets matching default change set (should be one, found {0:?})")]
    UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(Vec<ChangeSetId>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status_code = match &self {
            Self::ChangeSetApply(_) => StatusCode::CONFLICT,
            Self::DvuRootsNotEmpty(_) => StatusCode::PRECONDITION_FAILED,
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub type ChangeSetAPIError = Error;

type Result<T> = result::Result<T, Error>;

pub fn v2_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/apply", post(apply::apply))
        .route(
            "/request_approval",
            post(request_approval::request_approval),
        )
        .route(
            "/approve",
            post(approve::approve).layer(WorkspacePermissionLayer::new(
                state.clone(),
                permissions::Permission::Approve,
            )),
        )
        .route(
            "/reject",
            post(reject::reject).layer(WorkspacePermissionLayer::new(
                state.clone(),
                permissions::Permission::Approve,
            )),
        )
        .route(
            "/cancel_approval_request",
            post(cancel_approval_request::cancel_approval_request),
        )
        // Consider how we make it editable again after it's been rejected
        .route("/reopen", post(reopen::reopen))
        .route("/list", get(list::list_actionable))
        .route(
            "/force_apply",
            post(force_apply::force_apply).layer(WorkspacePermissionLayer::new(
                state.clone(),
                permissions::Permission::Approve,
            )),
        )
}
