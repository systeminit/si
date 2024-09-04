use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use chrono::{DateTime, Utc};
use dal::{
    func::runner::FuncRunnerError, workspace_snapshot::graph::WorkspaceSnapshotGraphDiscriminants,
    ChangeSet, ChangeSetId, ChangeSetStatus, UserPk, Workspace, WorkspacePk,
    WorkspaceSnapshotAddress,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::server::state::AppState;
use crate::server::{error, extract::AdminAccessBuilder};
use crate::service::ApiError;

mod get_snapshot;
mod kill_execution;
mod list_change_sets;
mod list_workspaces;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AdminAPIError {
    #[error("axum http error: {0}")]
    AxumHttp(#[from] axum::http::Error),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("change set {0} not found")]
    ChangeSetNotFound(ChangeSetId),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspaces error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
    #[error("change set {0} does not have a workspace snapshot address")]
    WorkspaceSnapshotAddressNotFound(ChangeSetId),
    #[error("workspace snapshot {0} for change set {1} could not be found in durable storage")]
    WorkspaceSnapshotNotFound(WorkspaceSnapshotAddress, ChangeSetId),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdminWorkspace {
    pub id: WorkspacePk,
    pub name: String,
    pub default_change_set_id: ChangeSetId,
    #[serde(flatten)]
    pub timestamp: dal::Timestamp,
    pub snapshot_version: WorkspaceSnapshotGraphDiscriminants,
    pub component_concurrency_limit: i32,
}

impl From<Workspace> for AdminWorkspace {
    fn from(value: Workspace) -> Self {
        Self {
            id: *value.pk(),
            name: value.name().to_owned(),
            default_change_set_id: value.default_change_set_id(),
            timestamp: value.timestamp().to_owned(),
            snapshot_version: value.snapshot_version(),
            component_concurrency_limit: value.component_concurrency_limit(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AdminChangeSet {
    pub id: ChangeSetId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name: String,
    pub status: ChangeSetStatus,
    pub base_change_set_id: Option<ChangeSetId>,
    pub workspace_snapshot_address: Option<WorkspaceSnapshotAddress>,
    pub workspace_id: Option<WorkspacePk>,
    pub merge_requested_by_user_id: Option<UserPk>,
}

impl From<ChangeSet> for AdminChangeSet {
    fn from(value: ChangeSet) -> Self {
        Self {
            id: value.id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            name: value.name,
            status: value.status,
            base_change_set_id: value.base_change_set_id,
            workspace_snapshot_address: value.workspace_snapshot_address,
            workspace_id: value.workspace_id,
            merge_requested_by_user_id: value.merge_requested_by_user_id,
        }
    }
}

impl IntoResponse for AdminAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            AdminAPIError::FuncRunner(FuncRunnerError::DoNotHavePermissionToKillExecution) => {
                StatusCode::UNAUTHORIZED
            }
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };
        error!(si.error.message = ?self.to_string());

        ApiError::new(status_code, self).into_response()
    }
}

pub type AdminAPIResult<T> = Result<T, AdminAPIError>;

pub fn v2_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/func/runs/:func_run_id/kill_execution",
            put(kill_execution::kill_execution),
        )
        .route("/workspaces", get(list_workspaces::list_workspaces))
        .route(
            "/workspaces/:workspace_pk/change_sets",
            get(list_change_sets::list_change_sets),
        )
        .route(
            "/workspaces/:workspace_pk/change_sets/:change_set_id/get_snapshot",
            get(get_snapshot::get_snapshot),
        )
        .route_layer(axum::middleware::from_extractor_with_state::<
            AdminAccessBuilder,
            AppState,
        >(state))
}
