use axum::Json;
use dal::{Component, ComponentId, StandardModel, SystemId, Visibility};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeResponse {
    pub success: bool,
}

pub async fn generate_code(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<GenerateCodeRequest>,
) -> ComponentResult<Json<GenerateCodeResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let system_id = request.system_id.unwrap_or_else(|| SystemId::NONE);
    ctx.enqueue_job(CodeGenerationJob::new(request.component_id, system_id)).await?;

    txns.commit().await?;

    Ok(Json(GenerateCodeResponse { success: true }))
}
