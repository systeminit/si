use axum::{extract::Path, Json};
use dal::{User, WorkspacePk};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::{AdminAPIResult, AdminUser};
use axum_util::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersForWorkspaceResponse {
    users: Vec<AdminUser>,
}

#[instrument(name = "admin.list_users_for_workspace", skip_all)]
pub async fn list_workspace_users(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path(workspace_pk): Path<WorkspacePk>,
) -> AdminAPIResult<Json<ListUsersForWorkspaceResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let users = User::list_members_for_workspace(&ctx, workspace_pk.to_string())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(ListUsersForWorkspaceResponse { users }))
}
