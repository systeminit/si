use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::node::NodeId;
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

    let position = NodePosition::upsert_by_node_id(
        &ctx,
        DiagramKind::Configuration,
        request.node_id,
        &request.x,
        &request.y,
    )
    .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(SetNodePositionResponse { position }))
}
