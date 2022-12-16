use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::diagram::DiagramError;
use axum::Json;
use dal::node::NodeId;
use dal::socket::SocketEdgeKind;
use dal::{DiagramKind, NodePosition, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub node_id: NodeId,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionResponse {
    pub position: NodePosition,
}

pub async fn set_node_position(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetNodePositionRequest>,
) -> DiagramResult<Json<SetNodePositionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let (width, height) = {
        let component = dal::Component::find_for_node(&ctx, request.node_id)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let sockets = component
            .schema_variant(&ctx)
            .await?
            .ok_or(DiagramError::SchemaVariantNotFound)?
            .sockets(&ctx)
            .await?;

        let mut size = (None, None);

        for s in sockets {
            // If component is a frame, we set the size as either the one from the request or the previous one
            // If we don't do it like this upsert_by_node_id will delete the size on None instead of keeping it as is
            if s.name() == "Frame" && *s.edge_kind() == SocketEdgeKind::ConfigurationInput {
                let node_position = NodePosition::list_for_node(&ctx, request.node_id)
                    .await?
                    .into_iter()
                    .find(|n| *n.diagram_kind() == DiagramKind::Configuration)
                    .ok_or(DiagramError::NodeNotFound(request.node_id))?;

                size = (
                    request
                        .width
                        .or_else(|| node_position.width().map(|v| v.to_string())),
                    request
                        .height
                        .or_else(|| node_position.height().map(|v| v.to_string())),
                );
                break;
            }
        }

        size
    };

    let position = NodePosition::upsert_by_node_id(
        &ctx,
        DiagramKind::Configuration,
        request.node_id,
        &request.x,
        &request.y,
        width,
        height,
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(SetNodePositionResponse { position }))
}
