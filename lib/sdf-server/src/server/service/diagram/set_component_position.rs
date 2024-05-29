use axum::{response::IntoResponse, Json};
use dal::component::ComponentGeometry;
use dal::{
    component::frame::Frame,
    diagram::{SummaryDiagramComponent, SummaryDiagramInferredEdge},
    ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SingleComponentGeometryUpdate {
    pub geometry: ComponentGeometry,
    pub detach: bool,
    pub new_parent: Option<ComponentId>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub data_by_component_id: HashMap<ComponentId, SingleComponentGeometryUpdate>,
}

pub async fn set_component_position(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetComponentPositionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut components: Vec<Component> = vec![];
    let mut diagram_inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];

    for (id, update) in request.data_by_component_id {
        let mut component = Component::get_by_id(&ctx, id).await?;

        if update.detach {
            Frame::orphan_child(&ctx, component.id()).await?;
            let payload: SummaryDiagramComponent =
                SummaryDiagramComponent::assemble(&ctx, &component).await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        } else if let Some(new_parent) = update.new_parent {
            Frame::upsert_parent(&ctx, component.id(), new_parent).await?;
            let payload: SummaryDiagramComponent =
                SummaryDiagramComponent::assemble(&ctx, &component).await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;

            // Queue new implicit edges to send to frontend
            {
                let component = Component::get_by_id(&ctx, new_parent).await?;
                for inferred_incoming_connection in
                    component.inferred_incoming_connections(&ctx).await?
                {
                    diagram_inferred_edges.push(SummaryDiagramInferredEdge::assemble(
                        inferred_incoming_connection,
                    )?)
                }
                for inferred_outgoing_connection in
                    component.inferred_outgoing_connections(&ctx).await?
                {
                    diagram_inferred_edges.push(SummaryDiagramInferredEdge::assemble(
                        inferred_outgoing_connection,
                    )?)
                }
            }
        }

        let (width, height) = {
            let mut size = (None, None);

            let component_type = component.get_type(&ctx).await?;

            if component_type != ComponentType::Component {
                size = (
                    update
                        .geometry
                        .width
                        .or_else(|| component.width().map(|v| v.to_string())),
                    update
                        .geometry
                        .height
                        .or_else(|| component.height().map(|v| v.to_string())),
                );
            }

            size
        };

        component
            .set_geometry(&ctx, update.geometry.x, update.geometry.y, width, height)
            .await?;
        components.push(component);
    }
    let user_id = ChangeSet::extract_userid_from_context(&ctx).await;

    WsEvent::set_component_position(&ctx, ctx.change_set_id(), components, user_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    WsEvent::upsert_inferred_edges(&ctx, diagram_inferred_edges)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(axum::body::Empty::new())?)
}
