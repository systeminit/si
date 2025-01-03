use axum::Json;
use dal::{User, Workspace};
use serde::{Deserialize, Serialize};

use super::{SessionError, SessionResult};
use crate::extract::{AccessBuilder, EndpointAuthorization, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub workspace: Workspace,
}

pub async fn restore_authentication(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    EndpointAuthorization {
        user, workspace_id, ..
    }: EndpointAuthorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let workspace = Workspace::get_by_pk(&ctx, &workspace_id)
        .await?
        .ok_or(SessionError::InvalidWorkspace(workspace_id))?;
    let reply = RestoreAuthenticationResponse { user, workspace };

    Ok(Json(reply))
}
