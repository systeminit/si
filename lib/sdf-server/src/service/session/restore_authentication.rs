use anyhow::Result;
use axum::Json;
use dal::{User, Workspace};
use serde::{Deserialize, Serialize};

use crate::extract::{v1::AccessBuilder, workspace::WorkspaceAuthorization, HandlerContext};

use super::SessionError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub workspace: Workspace,
}

pub async fn restore_authentication(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    WorkspaceAuthorization {
        user, workspace_id, ..
    }: WorkspaceAuthorization,
) -> Result<Json<RestoreAuthenticationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspace = Workspace::get_by_pk(&ctx, &workspace_id)
        .await?
        .ok_or(SessionError::InvalidWorkspace(workspace_id))?;
    let reply = RestoreAuthenticationResponse { user, workspace };

    Ok(Json(reply))
}
