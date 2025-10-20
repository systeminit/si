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
    InviteMemberRequest,
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
    post,
    path = "/management/workspaces/{workspace_id}/members",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier")
    ),
    request_body = InviteMemberRequest,
    tag = "workspace_management",
    summary = "Invite a new member to the workspace",
    responses(
        (status = 200, description = "Member invited successfully", body = Vec<Member>),
        (status = 400, description = "Bad Request - Invalid email format"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - User must be workspace owner or approver to invite members"),
        (status = 404, description = "Workspace not found or has been deleted"),
        (status = 409, description = "Conflict - User already invited, suspended, or other conflict"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(clippy::too_many_arguments)]
pub async fn invite_member(
    HandlerContext(builder): HandlerContext,
    validated_token: ValidatedToken,
    RawAccessToken(token): RawAccessToken,
    RequestUlidFromHeader(request_ulid): RequestUlidFromHeader,
    State(state): State<AppState>,
    Path(WorkspaceManagementRequestPath { workspace_id }): Path<WorkspaceManagementRequestPath>,
    tracker: PosthogEventTracker,
    payload: Result<Json<InviteMemberRequest>, axum::extract::rejection::JsonRejection>,
) -> WorkspaceManagementResult<Json<Vec<Member>>> {
    let Json(payload) = payload?;

    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{}/workspace/{}/members",
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
