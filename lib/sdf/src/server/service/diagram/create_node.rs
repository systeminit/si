use axum::Json;
use serde::{Deserialize, Serialize};

use dal::edge::EdgeKind;
use dal::frame::Frame;
use dal::node::NodeId;
use dal::socket::SocketEdgeKind;
use dal::{
    generate_name, node_position::NodePositionView, Component, Connection, DiagramKind,
    NodePosition, NodeTemplate, NodeView, Schema, SchemaId, Socket, StandardModel, Visibility,
    WorkspaceId,
};

use crate::server::extract::{AccessBuilder, HandlerContext};
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
    pub node: NodeView,
}

pub async fn create_node(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateNodeRequest>,
) -> DiagramResult<Json<CreateNodeResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Create the component.
    let name = generate_name();
    let schema = Schema::get_by_id(&ctx, &request.schema_id)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;
    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(DiagramError::SchemaVariantNotFound)?;
    let (component, node) = Component::new(&ctx, &name, *schema_variant_id).await?;
    let component_id = *component.id();

    // Create the node template.
    let node_template = NodeTemplate::new_for_schema(&ctx, request.schema_id).await?;

    // NOTE(nick): we currently assume all nodes created through this route are configuration nodes.
    let position = NodePosition::new(
        &ctx,
        *node.id(),
        DiagramKind::Configuration,
        request.x.clone(),
        request.y.clone(),
        Some("500".to_string()),
        Some("500".to_string()),
    )
    .await?;
    let positions = vec![NodePositionView::from(position)];
    let node_view = NodeView::new(name, &node, component_id, positions, node_template);

    // If we have a parent, we need to assign the new node to a frame and perform both the symbolic
    // and underlying ("actual") connections.
    if let Some(frame_node_id) = request.parent_id {
        Frame::connect(&ctx, frame_node_id, *node.id()).await?;
    }

    ctx.commit().await?;

    Ok(Json(CreateNodeResponse { node: node_view }))
}
