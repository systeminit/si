use axum::Json;
use dal::job::definition::DependentValuesUpdate;
use dal::{
    node::NodeId, AttributeReadContext, AttributeValue, Connection, ExternalProvider, Node,
    SocketId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAggregateProxyConnectionRequest {
    pub parent_node_id: NodeId,
    pub child_node_ids: Vec<NodeId>,
    pub from_socket_id: SocketId,
    pub to_socket_id: SocketId,
    pub from_node_id: NodeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAggregateProxyConnectionResponse {
    pub connections: Vec<Connection>,
}

pub async fn create_aggregate_proxy_connections(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateAggregateProxyConnectionRequest>,
) -> DiagramResult<Json<CreateAggregateProxyConnectionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let node = Node::get_by_id(&ctx, &request.from_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(request.from_node_id))?;

    let component = node
        .component(&ctx)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let from_socket_external_provider =
        ExternalProvider::find_for_socket(&ctx, request.from_socket_id)
            .await?
            .ok_or(DiagramError::ExternalProviderNotFoundForSocket(
                request.from_socket_id,
            ))?;

    let attribute_value_context = AttributeReadContext {
        external_provider_id: Some(*from_socket_external_provider.id()),
        component_id: Some(*component.id()),
        ..Default::default()
    };

    let attribute_value = AttributeValue::find_for_context(&ctx, attribute_value_context)
        .await?
        .ok_or(DiagramError::AttributeValueNotFoundForContext(
            attribute_value_context,
        ))?;

    let mut connections = Vec::new();

    let connection = Connection::new(
        &ctx,
        request.from_node_id,
        request.from_socket_id,
        request.parent_node_id,
        request.to_socket_id,
    )
    .await?;

    connections.push(connection);

    for child in request.child_node_ids {
        let connection = Connection::new(
            &ctx,
            request.from_node_id,
            request.from_socket_id,
            child,
            request.to_socket_id,
        )
        .await?;

        connections.push(connection);
    }

    ctx.enqueue_job(DependentValuesUpdate::new(&ctx, *attribute_value.id()))
        .await;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateAggregateProxyConnectionResponse { connections }))
}
