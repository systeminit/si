use axum::Json;
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{
    job::definition::{fix::Fix, Fixes},
    ComponentId, ConfirmationResolverId, FixExecutionBatch, StandardModel, Visibility,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixRunRequest {
    pub id: ConfirmationResolverId,
    pub component_id: ComponentId,
    pub action_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixesRunRequest {
    pub list: Vec<FixRunRequest>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixesRunResponse {
    success: bool,
}

pub async fn run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<FixesRunRequest>,
) -> FixResult<Json<FixesRunResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut fixes = Vec::with_capacity(request.list.len());
    for fix in request.list {
        fixes.push(Fix {
            confirmation_resolver_id: fix.id,
            component_id: fix.component_id,
            action: fix.action_name,
        });
    }

    let batch = FixExecutionBatch::new(&ctx).await?;

    ctx.enqueue_job(Fixes::new(&ctx, fixes, *batch.id())).await;

    ctx.commit().await?;

    Ok(Json(FixesRunResponse { success: true }))
}
