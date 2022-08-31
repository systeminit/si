use axum::{extract::Query, Json};
use dal::{SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::DevResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestRequest {
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestResponse {
    pub success: bool,
}

pub async fn test(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<TestRequest>,
) -> DevResult<Json<TestResponse>> {
    let txns = txns.start().await?;
    let _ctx = builder.build(request_ctx.build(request.visibility), &txns);
    let _system_id = request.system_id.unwrap_or(SystemId::NONE);
    txns.commit().await?;
    Ok(Json(TestResponse { success: true }))
}
