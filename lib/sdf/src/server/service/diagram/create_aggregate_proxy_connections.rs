use axum::Json;
use dal::edge::EdgeKind;
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
pub struct AggregateNodePayload {
    pub node_id: NodeId,
    pub is_parent: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateAggregateProxyConnectionRequest {
    pub to_node_ids: Vec<AggregateNodePayload>,
    pub to_socket_id: SocketId,
    pub from_node_ids: Vec<AggregateNodePayload>,
    pub from_socket_id: SocketId,
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

    let mut connections = Vec::new();

    for from_node in request.from_node_ids {
        for to_node in request.to_node_ids.iter() {
            let edge_kind = if from_node.is_parent || to_node.is_parent {
                EdgeKind::Symbolic
            } else {
                EdgeKind::Configuration
            };

            let connection = Connection::new(
                &ctx,
                from_node.node_id,
                request.from_socket_id,
                to_node.node_id,
                request.to_socket_id,
                edge_kind,
            )
            .await?;

            connections.push(connection);
        }

        let component = Node::get_by_id(&ctx, &from_node.node_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(from_node.node_id))?
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

        ctx.enqueue_job(DependentValuesUpdate::new(&ctx, *attribute_value.id()))
            .await;
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateAggregateProxyConnectionResponse { connections }))
}
