use axum::Json;
use serde::{Deserialize, Serialize};

use dal::edge::EdgeKind;
use dal::node::NodeId;
use dal::socket::SocketEdgeKind;
use dal::{
    generate_name, Component, ComponentId, Connection, DiagramKind, NodePosition, Schema, SchemaId,
    Socket, StandardModel, Visibility, WorkspaceId, WsEvent,
};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::diagram::connect_component_to_frame::connect_component_sockets_to_frame;
use crate::service::diagram::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub schema_id: SchemaId,
    pub parent_id: Option<NodeId>,
    pub x: String,
    pub y: String,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeResponse {
    pub component_id: ComponentId,
}

pub async fn create_node(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateNodeRequest>,
) -> DiagramResult<Json<CreateNodeResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let name = generate_name();
    let schema = Schema::get_by_id(&ctx, &request.schema_id)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let (component, node) = Component::new(&ctx, &name, *schema_variant_id).await?;

    // NOTE(nick): we currently assume all nodes created through this route are configuration nodes.
    NodePosition::new(
        &ctx,
        *node.id(),
        DiagramKind::Configuration,
        request.x.clone(),
        request.y.clone(),
        Some("500"),
        Some("500"),
    )
    .await?;

    if let Some(frame_id) = request.parent_id {
        let component_socket = Socket::find_frame_socket_for_node(
            &ctx,
            *node.id(),
            SocketEdgeKind::ConfigurationOutput,
        )
        .await?;
        let frame_socket =
            Socket::find_frame_socket_for_node(&ctx, frame_id, SocketEdgeKind::ConfigurationInput)
                .await?;

        let _connection = Connection::new(
            &ctx,
            *node.id(),
            *component_socket.id(),
            frame_id,
            *frame_socket.id(),
            EdgeKind::Symbolic,
        )
        .await?;

        connect_component_sockets_to_frame(&ctx, frame_id, *node.id()).await?;
    }

    // TODO(nick,theo): create a component-specific event once the endpoints are cleaner (i.e. we
    // can call routes with more precision).
    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(CreateNodeResponse {
        component_id: *component.id(),
    }))
}
