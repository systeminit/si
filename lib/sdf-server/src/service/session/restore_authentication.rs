use axum::Json;
use dal::{User, Workspace};
use serde::{Deserialize, Serialize};

use super::{SessionError, SessionResult};
use crate::extract::workspace::WorkspaceAuthorization;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub workspace: Workspace,
}

pub async fn restore_authentication(
    WorkspaceAuthorization {
        ctx,
        user,
        workspace_id,
        ..
    }: WorkspaceAuthorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let workspace = Workspace::get_by_pk(&ctx, &workspace_id)
        .await?
        .ok_or(SessionError::InvalidWorkspace(workspace_id))?;
    let reply = RestoreAuthenticationResponse { user, workspace };

    Ok(Json(reply))
}
