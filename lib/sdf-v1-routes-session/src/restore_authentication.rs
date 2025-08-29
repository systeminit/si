use axum::Json;
use dal::Workspace;
use sdf_extract::{
    HandlerContext,
    v1::AccessBuilder,
    workspace::WorkspaceAuthorization,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::User;

use crate::SessionResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub workspace: Workspace,
    pub user_workspace_flags: serde_json::Value,
}

pub async fn restore_authentication(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    WorkspaceAuthorization {
        user, workspace_id, ..
    }: WorkspaceAuthorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspace = Workspace::get_by_pk(&ctx, workspace_id).await?;

    let user_workspace_flags =
        User::get_flags_for_user_on_workspace(&ctx, user.pk(), workspace_id).await?;

    let reply = RestoreAuthenticationResponse {
        user,
        workspace,
        user_workspace_flags,
    };

    Ok(Json(reply))
}
