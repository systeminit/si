use std::collections::HashMap;

use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent};
use sdf_core::{force_change_set_response::ForceChangeSetResponse, tracking::track};
use sdf_extract::{v1::AccessBuilder, HandlerContext, PosthogClient};
use serde::{Deserialize, Serialize};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetTypeRequest {
    pub component_id: ComponentId,
    pub component_type: ComponentType,
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
        visibility,
    }): Json<SetTypeRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    Component::set_type_by_id(&ctx, component_id, component_type).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    let mut socket_map = HashMap::new();
    // PSA: when we call `set_type_by_id` we are not altering any geometries (e.g. turning a small component into a default 500x500 sized frame)
    // if we do alter those geometries, we need to send multiple geometries back over the wire (currently, we only support sending one)
    let payload = component
        .into_frontend_type(
            &ctx,
            None,
            component.change_status(&ctx).await?,
            &mut socket_map,
        )
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
