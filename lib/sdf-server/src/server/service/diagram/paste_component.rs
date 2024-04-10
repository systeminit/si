use axum::{extract::OriginalUri, http::uri::Uri};
use axum::{response::IntoResponse, Json};
use dal::{component::frame::Frame, ChangeSet, Component, ComponentId, DalContext, Visibility};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[allow(clippy::too_many_arguments)]
async fn paste_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    offset_x: f64,
    offset_y: f64,
    original_uri: &Uri,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<Component> {
    let original_comp = Component::get_by_id(ctx, component_id).await?;
    let pasted_comp = original_comp.copy_paste(ctx, (offset_x, offset_y)).await?;

    let schema = pasted_comp.schema(ctx).await?;
    track(
        posthog_client,
        ctx,
        original_uri,
        "paste_component",
        serde_json::json!({
            "component_id": pasted_comp.id(),
            "component_schema_name": schema.name(),
        }),
    );

    Ok(pasted_comp)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    pub offset_x: f64,
    pub offset_y: f64,
    pub new_parent_component_id: Option<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Paste a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn paste_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<PasteComponentsRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let maybe_force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let mut pasted_components_by_original = HashMap::new();
    for component_id in &request.component_ids {
        let posthog_client = PosthogClient(posthog_client.clone());
        let pasted_comp = paste_single_component(
            &ctx,
            *component_id,
            request.offset_x,
            request.offset_y,
            &original_uri,
            &posthog_client,
        )
        .await?;

        pasted_components_by_original.insert(component_id, pasted_comp);
    }

    for component_id in &request.component_ids {
        let component = Component::get_by_id(&ctx, *component_id).await?;

        let pasted_component =
            if let Some(component) = pasted_components_by_original.get(&component_id) {
                component
            } else {
                return Err(DiagramError::Paste);
            };

        if let Some(parent_id) = component.parent(&ctx).await? {
            if let Some(pasted_parent) = pasted_components_by_original.get(&parent_id) {
                Frame::attach_child_to_parent(&ctx, pasted_parent.id(), pasted_component.id())
                    .await?;
            };
        }

        let incoming_connections = component.incoming_connections(&ctx).await?;
        for connection in incoming_connections {
            if let Some(from_component) =
                pasted_components_by_original.get(&connection.from_component_id)
            {
                Component::connect(
                    &ctx,
                    from_component.id(),
                    connection.from_output_socket_id,
                    pasted_component.id(),
                    connection.to_input_socket_id,
                )
                .await?;
            }
        }

        if let Some(parent_id) = request.new_parent_component_id {
            Frame::attach_child_to_parent(&ctx, parent_id, pasted_component.id()).await?;
        }
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = maybe_force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    Ok(response.body(serde_json::to_string(&())?)?)
}
