use super::{WorkspaceError, WorkspaceResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::User;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InviteRequest {
    pub email: String,
}

pub async fn invite(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<InviteRequest>,
) -> WorkspaceResult<Json<()>> {
    let ctx = builder.build_head(request_ctx).await?;

    User::invite_to_workspace(
        &ctx,
        &request.email,
        ctx.tenancy()
            .workspace_pk()
            .ok_or(WorkspaceError::NoWorkspace)?,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(()))
}
