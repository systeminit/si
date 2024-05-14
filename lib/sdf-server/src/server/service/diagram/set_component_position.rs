use axum::Json;
use dal::{
    component::frame::Frame,
    diagram::{SummaryDiagramComponent, SummaryDiagramInferredEdge},
    ChangeSet, Component, ComponentId, ComponentType, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElementPositions {
    pub component_id: ComponentId,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub positions: Vec<ElementPositions>,
    pub detach: bool,
    pub new_parent: Option<ComponentId>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentPositionResponse {
    pub component: Component,
}

pub async fn set_component_position(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetComponentPositionRequest>,
) -> DiagramResult<Json<()>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut components: Vec<Component> = vec![];
    for element in request.positions {
        let mut component = Component::get_by_id(&ctx, element.component_id).await?;

        if request.detach {
            Frame::orphan_child(&ctx, component.id()).await?;
            let payload: SummaryDiagramComponent =
                SummaryDiagramComponent::assemble(&ctx, &component).await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        } else if let Some(new_parent) = request.new_parent {
            Frame::upsert_parent(&ctx, component.id(), new_parent).await?;
            let payload: SummaryDiagramComponent =
                SummaryDiagramComponent::assemble(&ctx, &component).await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }

        let (width, height) = {
            let mut size = (None, None);

            let component_type = component.get_type(&ctx).await?;

            if component_type != ComponentType::Component {
                size = (
                    element
                        .width
                        .or_else(|| component.width().map(|v| v.to_string())),
                    element
                        .height
                        .or_else(|| component.height().map(|v| v.to_string())),
                );
            }

            size
        };

        component
            .set_geometry(&ctx, element.x, element.y, width, height)
            .await?;
        components.push(component);
    }
    let user_id = ChangeSet::extract_userid_from_context(&ctx).await;

    WsEvent::set_component_position(&ctx, ctx.change_set_id(), components, user_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    if let Some(new_parent) = request.new_parent {
        let component = Component::get_by_id(&ctx, new_parent).await?;
        let mut diagram_inferred_edges: Vec<SummaryDiagramInferredEdge> = vec![];
        for inferred_incoming_connection in component.inferred_incoming_connections(&ctx).await? {
            diagram_inferred_edges.push(SummaryDiagramInferredEdge::assemble(
                inferred_incoming_connection,
            )?)
        }
        for inferred_outgoing_connection in component.inferred_outgoing_connections(&ctx).await? {
            diagram_inferred_edges.push(SummaryDiagramInferredEdge::assemble(
                inferred_outgoing_connection,
            )?)
        }

        WsEvent::upsert_inferred_edges(&ctx, diagram_inferred_edges)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(Json(()))
}
