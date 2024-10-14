use std::collections::HashMap;

use axum::{
    extract::{Host, OriginalUri},
    response::IntoResponse,
    Json,
};
use dal::{ChangeSet, Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetResourceIdRequest {
    pub component_id: ComponentId,
    pub resource_id: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_resource_id(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(SetResourceIdRequest {
        component_id,
        resource_id,
        visibility,
    }): Json<SetResourceIdRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    component.set_resource_id(&ctx, &resource_id).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(&ctx, component.change_status(&ctx).await?, &mut socket_map)
        .await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "set_component_name",
        serde_json::json!({
            "how": "/component/set_name",
            "component_id": component.id(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
