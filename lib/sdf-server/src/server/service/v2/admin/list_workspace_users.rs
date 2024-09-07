use axum::{extract::Path, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use dal::{User, WorkspacePk};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{AdminAPIResult, AdminUser};

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
) -> AdminAPIResult<impl IntoResponse> {
    let ctx = builder.build_head(access_builder).await?;

    let users = User::list_members_for_workspace(&ctx, workspace_pk.to_string())
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(ListUsersForWorkspaceResponse { users }))
}
