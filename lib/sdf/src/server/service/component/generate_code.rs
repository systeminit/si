use axum::Json;
use dal::{
    job::definition::component_post_processing::ComponentPostProcessing, Component, ComponentId,
    StandardModel, SystemId, Visibility,
};
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
    let system_id = request.system_id.unwrap_or_else(|| SystemId::from(-1));

    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;

    ctx.enqueue_job(ComponentPostProcessing::new(&ctx, *component.id(), system_id, None).await?)
        .await;
    txns.commit().await?;

    Ok(Json(GenerateCodeResponse { success: true }))
}
