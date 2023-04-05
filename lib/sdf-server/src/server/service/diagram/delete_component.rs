use axum::extract::OriginalUri;
use axum::Json;
use dal::{Component, ComponentId, StandardModel, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

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
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentRequest>,
) -> DiagramResult<Json<DeleteComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut comp = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let comp_schema = comp
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    comp.delete_and_propagate(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "delete_component",
        serde_json::json!({
            "component_id": request.component_id,
            "component_schema_name": comp_schema.name(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(DeleteComponentResponse { success: true }))
}
