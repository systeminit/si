use axum::extract::OriginalUri;
use axum::Json;
use dal::{Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramError;
use dal::standard_model::StandardModel;

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
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RestoreComponentRequest>,
) -> DiagramResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    Component::restore_and_propagate(&ctx, request.component_id).await?;

    let (component, schema) = {
        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        let component = Component::get_by_id(ctx_with_deleted, &request.component_id)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        (component, schema)
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "restore_component",
        serde_json::json!({
                    "component_id": component.id(),
                    "component_schema_name": schema.name(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
