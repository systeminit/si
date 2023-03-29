use axum::Json;

use dal::{job::definition::RefreshJob, Component, ComponentId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub component_id: Option<ComponentId>,
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

    let component_ids = if let Some(component_id) = request.component_id {
        vec![component_id]
    } else {
        Component::list(&ctx)
            .await?
            .into_iter()
            .map(|c| *c.id())
            .collect()
    };

    ctx.enqueue_job(RefreshJob::new(&ctx, component_ids)).await;

    ctx.commit().await?;

    Ok(Json(RefreshRequestResponse { success: true }))
}
