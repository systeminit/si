use super::{SessionResult, SessionError};
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};
use axum::Json;
use dal::Workspace;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDefaultsResponse {
    pub workspace: Workspace,
}

pub async fn get_defaults(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
) -> SessionResult<Json<GetDefaultsResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    let workspace = Workspace::get_by_pk(&ctx, &claim.workspace_pk)
        .await?
        .ok_or(SessionError::InvalidWorkspace(claim.workspace_pk))?;

    Ok(Json(GetDefaultsResponse { workspace }))
}
