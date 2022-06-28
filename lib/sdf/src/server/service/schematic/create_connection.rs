use axum::Json;
use dal::{
    attribute::value::DependentValuesAsyncTasks, node::NodeId, socket::SocketId,
    AttributeReadContext, AttributeValue, ComponentAsyncTasks, Connection, ExternalProviderId,
    InternalProviderId, Node, StandardModel, SystemId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{SchematicError, SchematicResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use telemetry::prelude::*;

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
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    let connection = Connection::new(
        &ctx,
        &request.head_node_id,
        &request.head_socket_id,
        request.head_internal_provider_id,
        &request.tail_node_id,
        &request.tail_socket_id,
        request.tail_external_provider_id,
    )
    .await?;

    // TODO: get the appropriate system id
    let system_id = SystemId::NONE;

    let node = Node::get_by_id(&ctx, &request.tail_node_id)
        .await?
        .ok_or(SchematicError::NodeNotFound(request.tail_node_id))?;

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

    let attribute_value = AttributeValue::find_for_context(
        &ctx,
        AttributeReadContext {
            component_id: Some(*component.id()),
            schema_variant_id: Some(*schema_variant.id()),
            schema_id: Some(*schema.id()),
            system_id: Some(system_id),
            external_provider_id: Some(request.tail_external_provider_id),
            ..Default::default()
        },
    )
    .await?;

    let task = attribute_value.map(|attribute_value| {
        DependentValuesAsyncTasks::new(
            Some(ComponentAsyncTasks::new(component, system_id)),
            Some(*attribute_value.id()),
        )
    });

    txns.commit().await?;

    if let Some(task) = task {
        let request_ctx = request_ctx.clone();
        let builder = builder.clone();
        tokio::task::spawn(async move {
            if let Err(err) = task.run(request_ctx, request.visibility, &builder).await {
                error!("Component async qualification check failed: {err}");
            }
        });
    }

    Ok(Json(CreateConnectionResponse { connection }))
}
