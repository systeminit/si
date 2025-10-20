use axum::{
    Json,
    extract::{
        Path,
        State,
    },
};
use sdf_extract::request::RawAccessToken;

use super::{
    AuthApiWorkspace,
    Workspace,
    WorkspaceManagementError,
    WorkspaceManagementRequestPath,
    WorkspaceManagementResult,
    handle_auth_api_error,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    get,
    path = "/management/workspaces/{workspace_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "workspace_management",
    summary = "Get the details of a workspace",
    responses(
        (status = 200, description = "Workspace retrieved successfully", body = Workspace),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User is not a member of this workspace"),
        (status = 404, description = "Workspace not found or has been deleted"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_workspace(
    RawAccessToken(token): RawAccessToken,
    State(state): State<AppState>,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    _tracker: PosthogEventTracker,
) -> WorkspaceManagementResult<Json<Workspace>> {
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
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

    let auth_workspace = res.json::<AuthApiWorkspace>().await?;

    // Return 404 if workspace is deleted
    if auth_workspace.deleted_at.is_some() {
        return Err(WorkspaceManagementError::AuthApiError {
            status: axum::http::StatusCode::NOT_FOUND,
            message: "Workspace has been deleted and is no longer accessible - contact support if you need help".to_string(),
        });
    }

    let workspace = Workspace::from(auth_workspace);

    Ok(Json(workspace))
}
