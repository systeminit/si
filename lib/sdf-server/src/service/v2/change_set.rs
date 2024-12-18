use std::result;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dal::{
    workspace_integrations::WorkspaceIntegration, ChangeSetId, ChangeSetStatus, DalContext,
    HistoryEventError, WorkspacePk, WsEventError,
};
use reqwest::Client;
use serde::Serialize;
use si_data_spicedb::SpiceDbError;
use thiserror::Error;

use crate::{middleware::WorkspacePermissionLayer, service::ApiError, AppState};

mod apply;
mod approve;
mod cancel_approval_request;
mod force_apply;
mod list;
mod reject;
mod rename;
mod reopen;
mod request_approval;

// NOTE(nick): move these to the above group and remove old modules once the feature flag has been removed;
mod approval_status;
mod approve_v2;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("change set approval error: {0}")]
    Approval(#[from] dal::change_set::approval::ChangeSetApprovalError),
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
    #[error("history event: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("spice db error: {0}")]
    SpiceDB(#[from] SpiceDbError),
    #[error("spicedb not found")]
    SpiceDBNotFound,
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("found an unexpected number of open change sets matching default change set (should be one, found {0:?})")]
    UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(Vec<ChangeSetId>),
    #[error("Failed to post to webhook: {0}")]
    Webhook(String),
    #[error("workspace integration error: {0}")]
    WorkspaceIntegrations(#[from] dal::workspace_integrations::WorkspaceIntegrationsError),
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

#[derive(Serialize)]
struct SlackMessage<'a> {
    text: &'a str,
}

pub async fn post_to_webhook(
    ctx: &DalContext,
    _workspace_id: WorkspacePk,
    message: &str,
) -> Result<()> {
    if let Some(integration) = WorkspaceIntegration::get_integrations_for_workspace_pk(ctx).await? {
        if let Some(webhook_url) = integration.slack_webhook_url() {
            let client = Client::new();
            let slack_message = SlackMessage { text: message };

            let response = client
                .post(webhook_url.clone())
                .json(&slack_message)
                .send()
                .await?;

            if response.status().is_success() {
                return Ok(());
            } else {
                return Err(Error::Webhook(webhook_url));
            }
        }
    }

    Ok(())
}

pub fn v2_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest(
            "/:change_set_id",
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
                .route(
                    "/force_apply",
                    post(force_apply::force_apply).layer(WorkspacePermissionLayer::new(
                        state.clone(),
                        permissions::Permission::Approve,
                    )),
                )
                .route("/rename", post(rename::rename))
                .route("/approval_status", get(approval_status::approval_status))
                .route("/approve_v2", post(approve_v2::approve)),
        )
        .route("/", get(list::list_actionable))
}
