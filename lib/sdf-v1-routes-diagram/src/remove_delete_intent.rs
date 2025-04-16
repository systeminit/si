use std::collections::HashMap;

use axum::{
    extract::{Host, OriginalUri},
    http::uri::Uri,
    Json,
};
use dal::{ChangeSet, Component, ComponentId, DalContext, Visibility, WsEvent};
use sdf_core::{force_change_set_response::ForceChangeSetResponse, tracking::track};
use sdf_extract::{v1::AccessBuilder, HandlerContext, PosthogClient};
use serde::{Deserialize, Serialize};

use super::DiagramResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

async fn remove_single_delete_intent(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    let comp = Component::get_by_id(ctx, component_id).await?;

    let comp_schema = comp.schema(ctx).await?;
    let comp = comp.set_to_delete(ctx, false).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "remove_delete_intent",
        serde_json::json!({
            "how": "/diagram/remove_delete_intent",
            "component_id": comp.id(),
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(())
}

async fn restore_component_from_base_change_set(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    Component::restore_from_base_change_set(ctx, component_id).await?;
    let comp = Component::get_by_id(ctx, component_id).await?;
    let comp_schema = comp.schema(ctx).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "restore_from_base_change_set",
        serde_json::json!({
            "how": "/diagram/remove_delete_intent",
            "component_id": component_id,
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoveDeleteIntentRequest {
    pub components: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Restore a set of [`Component`](dal::Component)s via their componentId. Creating change set if on head.
pub async fn remove_delete_intent(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<RemoveDeleteIntentRequest>,
) -> DiagramResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    for component_id in request.components.clone() {
        let maybe_component = Component::try_get_by_id(&ctx, component_id).await?;
        if maybe_component.is_some() {
            remove_single_delete_intent(
                &ctx,
                component_id,
                &original_uri,
                &host_name,
                &posthog_client,
            )
            .await?;
        } else {
            restore_component_from_base_change_set(
                &ctx,
                component_id,
                &original_uri,
                &host_name,
                &posthog_client,
            )
            .await?;
        }
    }

    let mut diagram_sockets = HashMap::new();
    for component_id in request.components {
        let component = Component::get_by_id(&ctx, component_id).await?;
        let payload = component
            .into_frontend_type(
                &ctx,
                None,
                component.change_status(&ctx).await?,
                &mut diagram_sockets,
            )
            .await?;
        WsEvent::component_updated(&ctx, payload)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
