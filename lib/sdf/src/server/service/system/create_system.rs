use axum::Json;
use dal::{System, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SystemResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSystemRequest {
    pub name: String,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSystemResponse {
    pub system: System,
}

pub async fn create_system(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateSystemRequest>,
) -> SystemResult<Json<CreateSystemResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let (system, _node) = System::new_with_node(&ctx, request.name, &request.workspace_id).await?;

    txns.commit().await?;
    Ok(Json(CreateSystemResponse { system }))
}
