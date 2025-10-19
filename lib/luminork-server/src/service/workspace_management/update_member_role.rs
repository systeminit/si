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
    UpdateMemberRoleRequest,
    WorkspaceManagementRequestPath,
    WorkspaceManagementResult,
    handle_auth_api_error,
    map_role_to_auth_api,
    sync_members,
    transform_members,
};
use crate::{
    AppState,
    extract::PosthogEventTracker,
};

#[utoipa::path(
    post,
    path = "/management/workspaces/{workspace_id}/update_member_access",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    request_body = UpdateMemberRoleRequest,
    tag = "workspace_management",
    summary = "Update a member's role in the workspace",
    responses(
        (status = 200, description = "Member role updated successfully", body = Vec<Member>),
        (status = 400, description = "Bad Request - Invalid userId or role format"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User must be workspace owner to update member roles"),
        (status = 404, description = "Workspace not found or has been deleted"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn update_member_role(
    HandlerContext(builder): HandlerContext,
    validated_token: ValidatedToken,
    RawAccessToken(token): RawAccessToken,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
    State(state): State<AppState>,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    tracker: PosthogEventTracker,
    payload: Result<Json<UpdateMemberRoleRequest>, axum::extract::rejection::JsonRejection>,
) -> WorkspaceManagementResult<Json<Vec<Member>>> {
    let Json(mut payload) = payload?;

    // Map COLLABORATOR to EDITOR for Auth API
    payload.role = map_role_to_auth_api(&payload.role);

    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{}/workspace/{}/membership",
            state.auth_api_url(),
            workspace_id
        ))
        .bearer_auth(token)
        .json(&payload)
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
