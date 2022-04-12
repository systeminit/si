use super::SchematicResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::node::NodeId;
use dal::{NodePosition, SchematicKind, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionRequest {
    pub workspace_id: WorkspaceId,
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
    Json(request): Json<SetNodePositionRequest>,
) -> SchematicResult<Json<SetNodePositionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

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

    txns.commit().await?;

    Ok(Json(SetNodePositionResponse { position }))
}
