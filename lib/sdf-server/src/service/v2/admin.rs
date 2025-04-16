use axum::{
    async_trait,
    extract::{DefaultBodyLimit, FromRequestParts},
    http::{request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use dal::{
    cached_module::CachedModuleError, func::runner::FuncRunnerError,
    workspace_snapshot::graph::WorkspaceSnapshotGraphDiscriminants, ChangeSet, ChangeSetId,
    ChangeSetStatus, UserPk, Workspace, WorkspacePk, WorkspaceSnapshotAddress,
};
use sdf_core::api_error::ApiError;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    extract::{
        bad_request, internal_error,
        request::{RequestUlidFromHeader, ValidatedToken},
        unauthorized_error, ErrorResponse, HandlerContext,
    },
    AppState,
};

mod get_snapshot;
mod kill_execution;
mod list_change_sets;
mod search_workspaces;
mod set_concurrency_limit;
mod set_snapshot;
mod update_module_cache;

// 1GB
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 1024;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AdminAPIError {
    #[error("axum http error: {0}")]
    AxumHttp(#[from] axum::http::Error),
    #[error("cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("multipart error: {0}")]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error("No multipart data found in request")]
    NoMultipartData,
    #[error("tokio join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
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
    pub component_concurrency_limit: Option<i32>,
}

impl From<Workspace> for AdminWorkspace {
    fn from(value: Workspace) -> Self {
        Self {
            id: *value.pk(),
            name: value.name().to_owned(),
            default_change_set_id: value.default_change_set_id(),
            timestamp: value.timestamp().to_owned(),
            snapshot_version: value.snapshot_version(),
            component_concurrency_limit: value.raw_component_concurrency_limit(),
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
    pub workspace_snapshot_address: WorkspaceSnapshotAddress,
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

        ApiError::new(status_code, self).into_response()
    }
}

pub type AdminAPIResult<T> = Result<T, AdminAPIError>;

pub fn v2_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/update_module_cache",
            post(update_module_cache::update_module_cache),
        )
        .route(
            "/func/runs/:func_run_id/kill_execution",
            put(kill_execution::kill_execution),
        )
        .route("/workspaces", get(search_workspaces::search_workspaces))
        .route(
            "/workspaces/:workspace_id/set_concurrency_limit",
            post(set_concurrency_limit::set_concurrency_limit),
        )
        .route(
            "/workspaces/:workspace_id/change_sets",
            get(list_change_sets::list_change_sets),
        )
        .route(
            "/workspaces/:workspace_id/change_sets/:change_set_id/get_snapshot",
            get(get_snapshot::get_snapshot),
        )
        .route(
            "/workspaces/:workspace_id/change_sets/:change_set_id/set_snapshot",
            post(set_snapshot::set_snapshot),
        )
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .route_layer(axum::middleware::from_fn_with_state(
            state,
            require_systeminit_user,
        ))
}

/// An admin-only DAL context. Only constructed if the user's email
/// is @systeminit.com during construction.
#[derive(Clone, derive_more::Deref, derive_more::Into)]
pub struct AdminUserContext(pub dal::DalContext);

async fn require_systeminit_user<B>(
    builder: HandlerContext,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
    token: ValidatedToken,
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, ErrorResponse> {
    let ctx = builder
        .build_without_workspace(
            token.history_actor(),
            request_ulid,
            token.authentication_method().map_err(bad_request)?,
        )
        .await
        .map_err(internal_error)?;

    // Check if the user is @systeminit.com
    let is_system_init = ctx
        .history_actor()
        .email_is_systeminit(&ctx)
        .await
        .map_err(internal_error)?;
    if !is_system_init {
        return Err(unauthorized_error("not admin user"));
    }

    // Stash the context in extensions
    request.extensions_mut().insert(AdminUserContext(ctx));
    Ok(next.run(request).await)
}

#[async_trait]
impl FromRequestParts<AppState> for AdminUserContext {
    type Rejection = ErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(parts
            .extensions
            .get::<AdminUserContext>()
            .ok_or_else(|| internal_error("must use AdminUserContext in an admin endpoint"))?
            .clone())
    }
}
