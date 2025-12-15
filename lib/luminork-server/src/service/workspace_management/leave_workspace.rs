use axum::{
    Json,
    extract::{
        Path,
        State,
    },
};
use sdf_extract::{
    HandlerContext,
    request::{
        RawAccessToken,
        RequestUlidFromHeader,
        ValidatedToken,
    },
};

use super::{
    Member,
    WorkspaceManagementRequestPath,
    WorkspaceManagementResult,
    handle_auth_api_error,
    sync_members,
    transform_members,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    delete,
    path = "/management/workspaces/{workspace_id}/leave",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    tag = "workspace_management",
    summary = "Leave a workspace (remove yourself)",
    responses(
        (status = 200, description = "Successfully left workspace", body = Vec<Member>),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - Cannot leave workspace (e.g., you are the owner)"),
        (status = 404, description = "Workspace not found, has been deleted, or user not found in workspace"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn leave_workspace(
    HandlerContext(builder): HandlerContext,
    validated_token: ValidatedToken,
    RawAccessToken(token): RawAccessToken,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
    State(state): State<AppState>,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    tracker: PosthogEventTracker,
) -> WorkspaceManagementResult<Json<Vec<Member>>> {
    let client = reqwest::Client::new();

    let res = client
        .delete(format!(
            "{}/workspace/{}/leave",
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

    // Sync members to DAL and SpiceDB
    // Auth API has already validated permissions at this point
    sync_members(
        &builder,
        &state,
        &workspace_id,
        &validated_token,
        request_ulid,
        &members,
        &tracker,
    )
    .await?;

    let transformed_members = transform_members(members);

    Ok(Json(transformed_members))
}
