use axum::Json;
use dal::{Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a [`Component`](dal::Component) via its componentId.
pub async fn restore_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<RestoreComponentRequest>,
) -> DiagramResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    Component::restore_and_propagate(&ctx, request.component_id).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
