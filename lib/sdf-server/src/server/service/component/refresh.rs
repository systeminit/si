use axum::Json;

use dal::{
    job::definition::RefreshJob, Component, ComponentId, StandardModel, Visibility, WsEvent,
};
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

    // If a component does not exist on head, we should just consider it "refreshed" right away.
    if let Some(component_id) = request.component_id {
        if Component::get_by_id(&ctx, &component_id).await?.is_none() {
            WsEvent::resource_refreshed(&ctx, component_id)
                .await?
                .publish_on_commit(&ctx)
                .await?;
            ctx.commit().await?;

            return Ok(Json(RefreshRequestResponse { success: true }));
        }
    }

    let component_ids = if let Some(component_id) = request.component_id {
        vec![component_id]
    } else {
        Component::list(&ctx)
            .await?
            .into_iter()
            .filter(|c| c.needs_destroy())
            .map(|c| *c.id())
            .collect()
    };

    ctx.enqueue_job(RefreshJob::new(&ctx, component_ids)).await;

    ctx.commit().await?;

    Ok(Json(RefreshRequestResponse { success: true }))
}
