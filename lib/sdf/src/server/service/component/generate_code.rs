use axum::Json;
use dal::{
    attribute::value::DependentValuesAsyncTasks, context::JobContent, Component, ComponentId,
    StandardModel, SystemId, Visibility,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use telemetry::prelude::*;

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
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;
    let async_tasks = component.build_async_tasks(&ctx, system_id).await?;
    ctx.enqueue_job(JobContent::ComponentPostProcessing(async_tasks))
        .await;
    txns.commit().await?;

    //tokio::task::spawn(async move {
    //    if let Err(err) = async_tasks
    //        .run(request_ctx, request.visibility, &builder)
    //        .await
    //    {
    //        error!("Component async task execution failed: {err}");
    //    }
    //});

    Ok(Json(GenerateCodeResponse { success: true }))
}
