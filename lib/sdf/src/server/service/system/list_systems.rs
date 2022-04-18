use axum::{extract::Query, Json};
use dal::{LabelList, System, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::SystemResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSystemsRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSystemsResponse {
    pub list: LabelList<SystemId>,
}

pub async fn list_systems(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSystemsRequest>,
) -> SystemResult<Json<ListSystemsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);
    let list = System::list_for_workspace(&ctx, &request.workspace_id).await?;
    Ok(Json(ListSystemsResponse { list }))
}
