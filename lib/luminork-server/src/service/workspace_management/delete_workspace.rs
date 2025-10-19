use axum::{
    extract::{
        Path,
        State,
    },
    http::StatusCode,
};
use sdf_extract::request::RawAccessToken;

use super::{
    WorkspaceManagementRequestPath,
    WorkspaceManagementResult,
    handle_auth_api_error,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    delete,
    path = "/management/workspaces/{workspace_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "workspace_management",
    summary = "Delete a workspace - please note, this is a soft delete and workspaces can be recovered",
    responses(
        (status = 204, description = "Workspace deleted successfully"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User must be workspace owner to delete workspace"),
        (status = 404, description = "Workspace not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn delete_workspace(
    RawAccessToken(token): RawAccessToken,
    State(state): State<AppState>,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    _tracker: PosthogEventTracker,
) -> WorkspaceManagementResult<StatusCode> {
    let client = reqwest::Client::new();

    let res = client
        .delete(format!(
            "{}/workspaces/{}",
            state.auth_api_url(),
            workspace_id
        ))
        .bearer_auth(token)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(res).await);
    }

    Ok(StatusCode::NO_CONTENT)
}
