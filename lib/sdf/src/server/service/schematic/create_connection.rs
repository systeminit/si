use axum::Json;
use dal::{
    node::NodeId, socket::SocketId, Connection, ExternalProviderId, InternalProviderId, Visibility,
    WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub head_node_id: NodeId,
    pub head_socket_id: SocketId,
    pub head_internal_provider_id: InternalProviderId,
    pub tail_node_id: NodeId,
    pub tail_socket_id: SocketId,
    pub tail_external_provider_id: ExternalProviderId,
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

    // TODO(nick): pass through the provider ids.
    let connection = Connection::new(
        &ctx,
        &request.head_node_id,
        &request.head_socket_id,
        Some(request.head_internal_provider_id),
        &request.tail_node_id,
        &request.tail_socket_id,
        Some(request.tail_external_provider_id),
    )
    .await?;

    txns.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
