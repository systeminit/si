use axum::Json;
use dal::{
    job::definition::DependentValuesUpdate, node::NodeId, socket::SocketId, AttributeReadContext,
    AttributeValue, Connection, ExternalProvider, Node, StandardModel, Visibility, WorkspaceId,
    WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
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
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<Json<CreateConnectionResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let connection = Connection::new(
        &ctx,
        request.from_node_id,
        request.from_socket_id,
        request.to_node_id,
        request.to_socket_id,
    )
    .await?;

    let node = Node::get_by_id(&ctx, &request.from_node_id)
        .await?
        .ok_or(DiagramError::NodeNotFound(request.from_node_id))?;

    let component = node
        .component(&ctx)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let schema_variant = component
        .schema_variant(&ctx)
        .await?
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let schema = schema_variant
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let from_socket_external_provider =
        ExternalProvider::find_for_socket(&ctx, request.from_socket_id)
            .await?
            .ok_or(DiagramError::ExternalProviderNotFoundForSocket(
                request.from_socket_id,
            ))?;

    let attribute_value_context = AttributeReadContext {
        component_id: Some(*component.id()),
        schema_variant_id: Some(*schema_variant.id()),
        schema_id: Some(*schema.id()),
        external_provider_id: Some(*from_socket_external_provider.id()),
        ..Default::default()
    };
    let attribute_value = AttributeValue::find_for_context(&ctx, attribute_value_context)
        .await?
        .ok_or(DiagramError::AttributeValueNotFoundForContext(
            attribute_value_context,
        ))?;

    ctx.enqueue_job(DependentValuesUpdate::new(&ctx, *attribute_value.id()))
        .await;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
