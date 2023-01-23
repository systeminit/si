use axum::Json;
use dal::edge::{EdgeKind, EdgeObjectId, VertexObjectKind};
use dal::frame::Frame;
use dal::job::definition::DependentValuesUpdate;
use dal::socket::SocketEdgeKind;
use dal::{
    node::NodeId, AttributeReadContext, AttributeValue, Component, Connection, DalContext, Edge,
    EdgeError, ExternalProvider, InternalProvider, InternalProviderId, Node, PropId, StandardModel,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionRequest {
    pub child_node_id: NodeId,
    pub parent_node_id: NodeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFrameConnectionResponse {
    pub connection: Connection,
}

/// Create a [`Connection`](dal::Connection) with a _to_ [`Socket`](dal::Socket) and
/// [`Node`](dal::Node) and a _from_ [`Socket`](dal::Socket) and [`Node`](dal::Node).
pub async fn connect_component_to_frame(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateFrameConnectionRequest>,
) -> DiagramResult<Json<CreateFrameConnectionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let connection = Frame::connect(&ctx, request.parent_node_id, request.child_node_id).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(CreateFrameConnectionResponse { connection }))
}
