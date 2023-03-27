use axum::Json;
use dal::{Component, ComponentId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentResponse {
    pub success: bool,
}

/// Delete a [`Component`](dal::Component) via its componentId.
pub async fn delete_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeleteComponentRequest>,
) -> DiagramResult<Json<DeleteComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut comp = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    comp.delete_and_propagate(&ctx).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(DeleteComponentResponse { success: true }))
}
