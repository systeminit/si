use std::collections::HashMap;

use super::ComponentResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::diagram::geometry::RawGeometry;
use dal::{ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetTypeRequest {
    pub component_id: ComponentId,
    pub component_type: ComponentType,
    pub overridden_geometry: Option<RawGeometry>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_type(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(SetTypeRequest {
        component_id,
        component_type,
        overridden_geometry,
        visibility,
    }): Json<SetTypeRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    Component::set_type_by_id(&ctx, component_id, component_type).await?;
    let mut component = Component::get_by_id(&ctx, component_id).await?;

    if let Some(geometry) = overridden_geometry {
        component
            .set_geometry(
                &ctx,
                geometry.x,
                geometry.y,
                geometry.width,
                geometry.height,
            )
            .await?;
    }

    let component = Component::get_by_id(&ctx, component_id).await?;
    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(&ctx, component.change_status(&ctx).await?, &mut socket_map)
        .await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let component_schema = component.schema(&ctx).await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "set_component_type",
        serde_json::json!({
            "how": "/component/set_type",
            "component_id": component.id(),
            "component_schema_name": component_schema.name(),
            "new_component_type": component_type,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
