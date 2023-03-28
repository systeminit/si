use axum::Json;

use dal::{job::definition::RefreshJob, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub component_id: ComponentId,
    pub value: Option<serde_json::Value>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequestResponse {
    success: bool,
}

pub async fn refresh(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RefreshRequest>,
) -> ComponentResult<Json<RefreshRequestResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    ctx.enqueue_job(RefreshJob::new(&ctx, request.component_id))
        .await;

    ctx.commit().await?;

    Ok(Json(RefreshRequestResponse { success: true }))
}
