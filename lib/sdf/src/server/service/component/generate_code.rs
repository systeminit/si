use axum::Json;
use dal::{job::definition::CodeGeneration, Component, ComponentId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateCodeResponse {
    pub success: bool,
}

pub async fn generate_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<GenerateCodeRequest>,
) -> ComponentResult<Json<GenerateCodeResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;

    ctx.enqueue_job(CodeGeneration::new(&ctx, *component.id()).await?)
        .await;
    ctx.commit().await?;

    Ok(Json(GenerateCodeResponse { success: true }))
}
