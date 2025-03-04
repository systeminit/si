use std::collections::HashMap;

use super::DiagramResult;
use crate::{
    extract::{v1::AccessBuilder, HandlerContext},
    service::force_change_set_response::ForceChangeSetResponse,
};
use axum::Json;
use dal::diagram::view::View;
use dal::{
    component::{frame::Frame, InferredConnection},
    diagram::SummaryDiagramInferredEdge,
    ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::{RawGeometry, StringGeometry};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SingleComponentGeometryUpdate {
    pub geometry: StringGeometry,
    pub detach: bool,
    pub new_parent: Option<ComponentId>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub data_by_component_id: HashMap<ComponentId, SingleComponentGeometryUpdate>,
    pub client_ulid: Ulid,
    pub request_ulid: Ulid,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionResponse {
    pub request_ulid: Ulid,
}

pub async fn set_component_position(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetComponentPositionRequest>,
) -> DiagramResult<ForceChangeSetResponse<SetComponentPositionResponse>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut components: Vec<Component> = vec![];
    let mut diagram_inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];

    let mut socket_map = HashMap::new();
    let mut geometry_list = vec![];
    for (id, update) in request.data_by_component_id {
        let mut component = Component::get_by_id(&ctx, id).await?;

        let mut component_updated = false;

        if update.detach {
            Frame::orphan_child(&ctx, component.id()).await?;
            component_updated = true;
        } else if let Some(new_parent) = update.new_parent {
            Frame::upsert_parent(&ctx, component.id(), new_parent).await?;
            component_updated = true;

            // Queue new implicit edges to send to frontend
            {
                let component = Component::get_by_id(&ctx, new_parent).await?;
                let workspace_snapshot = ctx.workspace_snapshot()?;
                let mut inferred_connection_graph =
                    workspace_snapshot.inferred_connection_graph(&ctx).await?;
                for inferred_connection in inferred_connection_graph
                    .inferred_connections_for_component_stack(&ctx, component.id())
                    .await?
                {
                    if inferred_connection.source_component_id == component.id()
                        || inferred_connection.destination_component_id == component.id()
                    {
                        let to_delete = !Component::should_data_flow_between_components(
                            &ctx,
                            inferred_connection.destination_component_id,
                            inferred_connection.source_component_id,
                        )
                        .await?;
                        let inferred_stack_connection = InferredConnection {
                            to_component_id: inferred_connection.destination_component_id,
                            to_input_socket_id: inferred_connection.input_socket_id,
                            from_component_id: inferred_connection.source_component_id,
                            from_output_socket_id: inferred_connection.output_socket_id,
                            to_delete,
                        };
                        diagram_inferred_edges.push(SummaryDiagramInferredEdge::assemble(
                            inferred_stack_connection,
                        ))
                    }
                }
            }
        }

        if component_updated {
            let payload = component
                .into_frontend_type_for_default_view(
                    &ctx,
                    component.change_status(&ctx).await?,
                    &mut socket_map,
                )
                .await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }

        let default_view_id = View::get_id_for_default(&ctx).await?;

        let geometry = component.geometry(&ctx, default_view_id).await?;
        let new_geometry: RawGeometry = update.geometry.try_into()?;

        let (width, height) = {
            let mut size = (None, None);

            let component_type = component.get_type(&ctx).await?;

            if component_type != ComponentType::Component {
                size = (
                    new_geometry.width.or_else(|| geometry.width()),
                    new_geometry.height.or_else(|| geometry.height()),
                );
            }

            size
        };

        component
            .set_geometry(
                &ctx,
                default_view_id,
                new_geometry.x,
                new_geometry.y,
                width,
                height,
            )
            .await?;
        components.push(component);

        geometry_list.push((id.into(), new_geometry))
    }

    let view_id = View::get_id_for_default(&ctx).await?;

    WsEvent::set_component_position(
        &ctx,
        ctx.change_set_id(),
        view_id,
        geometry_list,
        Some(request.client_ulid),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    if !diagram_inferred_edges.is_empty() {
        WsEvent::upsert_inferred_edges(&ctx, diagram_inferred_edges)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        SetComponentPositionResponse {
            request_ulid: request.request_ulid,
        },
    ))
}
