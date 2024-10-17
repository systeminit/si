use super::ViewResult;
use crate::{
    extract::{AccessBuilder, HandlerContext},
    service::force_change_set_response::ForceChangeSetResponse,
};
use axum::extract::Path;
use axum::Json;
use dal::diagram::view::ViewId;
use dal::{
    component::{frame::Frame, InferredConnection},
    diagram::SummaryDiagramInferredEdge,
    ChangeSet, ChangeSetId, Component, ComponentId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionRequest {
    pub parent_id_by_component_id: HashMap<ComponentId, Option<ComponentId>>,
    pub client_ulid: Ulid,
    pub request_ulid: Ulid,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionResponse {
    pub request_ulid: Ulid,
}

// TODO move this to outside of the view controller
pub async fn set_component_parent(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, _view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<SetComponentPositionRequest>,
) -> ViewResult<ForceChangeSetResponse<SetComponentPositionResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut diagram_inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];

    let mut socket_map = HashMap::new();
    for (id, maybe_new_parent) in request.parent_id_by_component_id {
        let component = Component::get_by_id(&ctx, id).await?;

        if let Some(new_parent) = maybe_new_parent {
            Frame::upsert_parent(&ctx, component.id(), new_parent).await?;

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
                        )?)
                    }
                }
            }
        } else {
            Frame::orphan_child(&ctx, component.id()).await?;
        }

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
    }

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
