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
    SchemaError,
    SchemaVariantError,
    SecretError,
    WorkspacePk,
    WorkspaceSnapshotAddress,
    WsEventError,
    diagram::DiagramError,
    prop::PropError,
    property_editor::PropertyEditorError,
    workspace_integrations::WorkspaceIntegration,
    workspace_snapshot::dependent_value_root::DependentValueRootError,
};
use futures_lite::StreamExt;
use reqwest::Client;
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
    dal_wrapper::DalWrapperError,
    index::IndexError,
};
use serde::Serialize;
use si_data_spicedb::SpiceDbError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::middleware::WorkspacePermissionLayer;

mod apply;
mod approval_status;
mod approve;
mod cancel_approval_request;
mod create;
mod create_initialize_apply;
mod force_apply;
mod list;
mod rename;
mod reopen;
mod request_approval;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum Error {
    #[error("attributes error: {0}")]
    Attributes(#[from] dal::attribute::attributes::AttributesError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] dal::ChangeSetApplyError),
    #[error("change set approval error: {0}")]
    ChangeSetApproval(#[from] dal::change_set::approval::ChangeSetApprovalError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal wrapper error: {0}")]
    DalWrapper(#[from] sdf_core::dal_wrapper::DalWrapperError),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("deserializing mv index data error: {0}")]
    DeserializingMvIndexData(#[source] serde_json::Error),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
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
    #[error("index not found after fresh rebuild; workspace_pk={0}, change_set_id={1}")]
    IndexNotFoundAfterFreshBuild(WorkspacePk, ChangeSetId),
    #[error("index not found after rebuild; workspace_pk={0}, change_set_id={1}")]
    IndexNotFoundAfterRebuild(WorkspacePk, ChangeSetId),
    #[error("inferred connection graph error: {0}")]
    InferredConnectionGraph(
        #[from] dal::component::inferred_connection_graph::InferredConnectionGraphError,
    ),
    #[error("item with checksum not found; workspace_pk={0}, change_set_id={1}, kind={2}")]
    ItemWithChecksumNotFound(WorkspacePk, ChangeSetId, String),
    #[error("latest item not found; workspace_pk={0}, change_set_id={1}, kind={2}")]
    LatestItemNotFound(WorkspacePk, ChangeSetId, String),
    #[error("materialized view error: {0}")]
    MaterializedView(#[from] Box<dal_materialized_views::Error>),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("property editor error: {0}")]
    PropertyEditor(#[from] PropertyEditorError),
    #[error("http error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] dal::slow_rt::SlowRuntimeError),
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

impl From<dal_materialized_views::Error> for Error {
    fn from(error: dal_materialized_views::Error) -> Self {
        Box::new(error).into()
    }
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
            Error::ChangeSet(dal::ChangeSetError::CantRenameHeadChangeSet) => {
                (StatusCode::PRECONDITION_FAILED, None)
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

const WATCH_INDEX_TIMEOUT: Duration = Duration::from_secs(30);

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
        .route(
            "/create_initialize_apply",
            post(create_initialize_apply::create_initialize_apply),
        )
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

#[instrument(
    level = "info",
    name = "sdf.change_set.create_index_for_new_change_set_and_watch",
    skip_all,
    fields(
        si.edda_request.id = Empty
    )
)]
pub async fn create_index_for_new_change_set_and_watch(
    frigg: &frigg::FriggStore,
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    base_change_set_id: ChangeSetId,
    to_snapshot_address: WorkspaceSnapshotAddress,
) -> Result<bool> {
    let span = Span::current();
    let mut watch = frigg
        .watch_change_set_index(workspace_pk, change_set_id)
        .await?;
    let request_id = edda_client
        .new_change_set(
            workspace_pk,
            change_set_id,
            base_change_set_id,
            to_snapshot_address,
        )
        .await?;
    span.record("si.edda_request.id", request_id.to_string());

    let timeout = WATCH_INDEX_TIMEOUT;
    tokio::select! {
        _ = tokio::time::sleep(timeout) => {
            info!("timed out waiting for new change set index to be created");
            Ok(false)
        },
        _ = watch.next() => Ok(true)
    }
}

#[instrument(
    level = "info",
    name = "sdf.change_set.create_index_for_new_change_set",
    skip_all,
    fields(
        si.edda_request.id = Empty
    )
)]
pub async fn create_index_for_new_change_set(
    edda_client: &edda_client::EddaClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    base_change_set_id: ChangeSetId,
    to_snapshot_address: WorkspaceSnapshotAddress,
) -> Result<()> {
    let span = Span::current();
    let request_id = edda_client
        .new_change_set(
            workspace_pk,
            change_set_id,
            base_change_set_id,
            to_snapshot_address,
        )
        .await?;
    span.record("si.edda_request.id", request_id.to_string());

    Ok(())
}
