use std::{
    result,
    time::Duration,
};

use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        get,
        post,
    },
};
use dal::{
    ChangeSetId,
    DalContext,
    WorkspacePk,
    WsEventError,
    workspace_integrations::WorkspaceIntegration,
    workspace_snapshot::dependent_value_root::DependentValueRootError,
};
use reqwest::Client;
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
    dal_wrapper::DalWrapperError,
};
use serde::Serialize;
use si_data_spicedb::SpiceDbError;
use telemetry::prelude::*;
use thiserror::Error;

use super::index::IndexError;
use crate::middleware::WorkspacePermissionLayer;

mod apply;
mod approval_status;
mod approve;
mod cancel_approval_request;
mod create;
mod force_apply;
mod list;
mod rename;
mod reopen;
mod request_approval;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("change set approval error: {0}")]
    ChangeSetApproval(#[from] dal::change_set::approval::ChangeSetApprovalError),
    #[error("dal wrapper error: {0}")]
    DalWrapper(#[from] sdf_core::dal_wrapper::DalWrapperError),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("deserializing mv index data error: {0}")]
    DeserializingMvIndexData(#[source] serde_json::Error),
    #[error("dvu roots are not empty for change set: {0}")]
    DvuRootsNotEmpty(ChangeSetId),
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("frigg error: {0}")]
    Frigg(#[from] frigg::FriggError),
    #[error("index error: {0}")]
    Index(#[from] IndexError),
    #[error("index not found; workspace_pk={0}, change_set_id={1}")]
    IndexNotFound(WorkspacePk, ChangeSetId),
    #[error("index not found after rebuild; workspace_pk={0}, change_set_id={1}")]
    IndexNotFoundAfterFreshBuild(WorkspacePk, ChangeSetId),
    #[error("index not found after rebuild; workspace_pk={0}, change_set_id={1}")]
    IndexNotFoundAfterRebuild(WorkspacePk, ChangeSetId),
    #[error("item with checksum not found; workspace_pk={0}, change_set_id={1}, kind={2}")]
    ItemWithChecksumNotFound(WorkspacePk, ChangeSetId, String),
    #[error("latest item not found; workspace_pk={0}, change_set_id={1}, kind={2}")]
    LatestItemNotFound(WorkspacePk, ChangeSetId, String),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("spice db error: {0}")]
    SpiceDB(#[from] SpiceDbError),
    #[error("spicedb client not found")]
    SpiceDBClientNotFound,
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error(
        "found an unexpected number of open change sets matching default change set (should be one, found {0:?})"
    )]
    UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(Vec<ChangeSetId>),
    #[error("timed out when watching index with duration: {0:?}")]
    WatchIndexTimeout(Duration),
    #[error("workspace integration error: {0}")]
    WorkspaceIntegrations(#[from] dal::workspace_integrations::WorkspaceIntegrationsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status_code, maybe_error_override) = match &self {
            Self::DalWrapper(DalWrapperError::ApplyWithUnsatisfiedRequirements(_)) => (
                StatusCode::FORBIDDEN,
                Some(
                    "Cannot apply change set with unsatisfied requirements. Please try again."
                        .to_string(),
                ),
            ),
            Self::ChangeSetApply(_) => (StatusCode::CONFLICT, None),
            Self::DvuRootsNotEmpty(_) => (StatusCode::PRECONDITION_FAILED, None),
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                (StatusCode::FORBIDDEN, None)
            }
            _ => (ApiError::DEFAULT_ERROR_STATUS_CODE, None),
        };

        ApiError::new(
            status_code,
            maybe_error_override.unwrap_or(self.to_string()),
        )
        .into_response()
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
    workspace_id: WorkspacePk,
    message: &str,
) -> Result<()> {
    if let Some(integration) = WorkspaceIntegration::get_integrations_for_workspace_pk(ctx).await? {
        if let Some(webhook_url) = integration.slack_webhook_url() {
            let client = Client::new();
            let slack_message = SlackMessage { text: message };

            match client
                .post(webhook_url.clone())
                .json(&slack_message)
                .send()
                .await
            {
                Ok(response) if !response.status().is_success() => {
                    info!(
                        "Failed to post to Slack webhook for workspace {} to URL {}, status: {}",
                        workspace_id,
                        webhook_url,
                        response.status()
                    );
                }
                Err(err) => {
                    info!(
                        "Error posting to Slack webhook for workspace {} to URL {}: {}",
                        workspace_id, webhook_url, err
                    );
                }
                _ => {}
            }
        }
    }

    Ok(())
}

pub fn change_sets_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list::list_actionable))
        .route("/create_change_set", post(create::create_change_set))
}

pub fn change_set_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/apply", post(apply::apply))
        .route("/approval_status", get(approval_status::approval_status))
        .route("/approve", post(approve::approve))
        .route(
            "/cancel_approval_request",
            post(cancel_approval_request::cancel_approval_request),
        )
        .route(
            "/force_apply",
            post(force_apply::force_apply).layer(WorkspacePermissionLayer::new(
                state.clone(),
                permissions::Permission::Approve,
            )),
        )
        .route("/rename", post(rename::rename))
        // Consider how we make it editable again after it's been rejected
        .route("/reopen", post(reopen::reopen))
        .route(
            "/request_approval",
            post(request_approval::request_approval),
        )
        .nest("/index", super::index::v2_change_set_routes())
}
