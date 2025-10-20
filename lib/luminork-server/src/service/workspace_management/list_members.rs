use axum::{
    Json,
    extract::{
        Path,
        State,
    },
};
use sdf_extract::request::RawAccessToken;

use super::{
    Member,
    WorkspaceManagementRequestPath,
    WorkspaceManagementResult,
    handle_auth_api_error,
    transform_members,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    get,
    path = "/management/workspaces/{workspace_id}/members",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "workspace_management",
    summary = "List all members of a workspace",
    responses(
        (status = 200, description = "Members listed successfully", body = Vec<Member>),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User is not a member of this workspace"),
        (status = 404, description = "Workspace not found or has been deleted"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn list_members(
    RawAccessToken(token): RawAccessToken,
    State(state): State<AppState>,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    _tracker: PosthogEventTracker,
) -> WorkspaceManagementResult<Json<Vec<Member>>> {
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
            "{}/workspace/{}/members",
            state.auth_api_url(),
            workspace_id
        ))
        .bearer_auth(token)
        .send()
        .await?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(handle_auth_api_error(res).await);
    }

    let members = res.json::<Vec<Member>>().await?;

    // Transform members to map EDITOR role to COLLABORATOR
    let transformed_members = transform_members(members);

    Ok(Json(transformed_members))
}
