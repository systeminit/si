use axum::Json;
use dal::{
    job::definition::DependentValuesUpdate, node::NodeId, socket::SocketId, AttributeReadContext,
    AttributeValue, Connection, ExternalProvider, Node, StandardModel, SystemId, Visibility,
    WorkspaceId, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{SchematicError, SchematicResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub from_node_id: NodeId,
    pub from_socket_id: SocketId,
    pub to_node_id: NodeId,
    pub to_socket_id: SocketId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub connection: Connection,
}

/// Create a [`Connection`](dal::Connection) with a _to_ [`Socket`](dal::Socket) and
/// [`Node`](dal::Node) and a _from_ [`Socket`](dal::Socket) and [`Node`](dal::Node).
pub async fn create_connection(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateConnectionRequest>,
) -> SchematicResult<Json<CreateConnectionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let connection = Connection::new(
        &ctx,
        request.from_node_id,
        request.from_socket_id,
        request.to_node_id,
        request.to_socket_id,
    )
    .await?;

    let system_id = SystemId::NONE;

    let node = Node::get_by_id(&ctx, &request.from_node_id)
        .await?
        .ok_or(SchematicError::NodeNotFound(request.from_node_id))?;

    let component = node
        .component(&ctx)
        .await?
        .ok_or(SchematicError::ComponentNotFound)?;

    let schema_variant = component
        .schema_variant(&ctx)
        .await?
        .ok_or(SchematicError::SchemaVariantNotFound)?;

    let schema = schema_variant
        .schema(&ctx)
        .await?
        .ok_or(SchematicError::SchemaNotFound)?;

    let from_socket_external_provider =
        ExternalProvider::find_for_socket(&ctx, request.from_socket_id)
            .await?
            .ok_or(SchematicError::ExternalProviderNotFoundForSocket(
                request.from_socket_id,
            ))?;

    let attribute_value_context = AttributeReadContext {
        component_id: Some(*component.id()),
        schema_variant_id: Some(*schema_variant.id()),
        schema_id: Some(*schema.id()),
        system_id: Some(system_id),
        external_provider_id: Some(*from_socket_external_provider.id()),
        ..Default::default()
    };
    let attribute_value = AttributeValue::find_for_context(&ctx, attribute_value_context)
        .await?
        .ok_or(SchematicError::AttributeValueNotFoundForContext(
            attribute_value_context,
        ))?;

    ctx.enqueue_job(DependentValuesUpdate::new(&ctx, *attribute_value.id()))
        .await;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    txns.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
