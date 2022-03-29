use super::SchematicResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::node::NodeId;
use dal::socket::SocketId;
use dal::{Connection, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub head_node_id: NodeId,
    pub head_socket_id: SocketId,
    pub tail_node_id: NodeId,
    pub tail_socket_id: SocketId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub connection: Connection,
}

pub async fn create_connection(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateConnectionRequest>,
) -> SchematicResult<Json<CreateConnectionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let connection = Connection::new(
        &ctx,
        &request.head_node_id,
        &request.head_socket_id,
        &request.tail_node_id,
        &request.tail_socket_id,
    )
    .await?;

    txns.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
