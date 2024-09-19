use std::collections::HashMap;

use axum::{
    extract::{Host, OriginalUri},
    response::IntoResponse,
    Json,
};
use dal::{change_status::ChangeStatus, ChangeSet, Component, ComponentId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    pub component_id: ComponentId,
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_name(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(SetNameRequest {
        component_id,
        name,
        visibility,
    }): Json<SetNameRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    component.set_name(&ctx, &name).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    // TODO: We'll want to figure out whether this component is Added/Modified, depending on
    // whether it existed in the base change set already or not.
    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(&ctx, ChangeStatus::Unmodified, &mut socket_map)
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
