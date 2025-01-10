use axum::{extract::Path, Json};
use dal::{Tenancy, User, WorkspacePk};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::service::v2::admin::{AdminAPIResult, AdminUser, AdminUserContext};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListUsersForWorkspaceResponse {
    users: Vec<AdminUser>,
}

#[instrument(name = "admin.list_users_for_workspace", skip_all)]
pub async fn list_workspace_users(
    AdminUserContext(mut ctx): AdminUserContext,
    Path(workspace_id): Path<WorkspacePk>,
) -> AdminAPIResult<Json<ListUsersForWorkspaceResponse>> {
    ctx.update_tenancy(Tenancy::new(workspace_id));

    let users = User::list_members_for_workspace(&ctx, workspace_id.to_string())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(ListUsersForWorkspaceResponse { users }))
}
