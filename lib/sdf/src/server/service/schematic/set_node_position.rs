use super::{SchematicError, SchematicResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::node::NodeId;
use dal::{
    Node, NodeKind, NodePosition, SchematicKind, StandardModel, SystemId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
    pub node_id: NodeId,
    pub schematic_kind: SchematicKind,
    pub system_id: Option<SystemId>,
    pub deployment_node_id: Option<NodeId>,
    pub x: String,
    pub y: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionResponse {
    pub position: NodePosition,
}

pub async fn set_node_position(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(mut request): Json<SetNodePositionRequest>,
) -> SchematicResult<Json<SetNodePositionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let node = Node::find_by_attr(&ctx, "kind", &NodeKind::Deployment.as_ref())
        .await?
        .pop()
        .ok_or(SchematicError::InvalidSchematicKindParentNodeIdPair(
            request.schematic_kind,
            request.deployment_node_id,
        ))?;
    request.deployment_node_id = Some(*node.id());
    let position = NodePosition::upsert_by_node_id(
        &ctx,
        request.schematic_kind,
        request.system_id,
        request.deployment_node_id,
        request.node_id,
        &request.x,
        &request.y,
    )
    .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    txns.commit().await?;

    Ok(Json(SetNodePositionResponse { position }))
}
