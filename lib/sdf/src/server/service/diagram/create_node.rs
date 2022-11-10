use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::diagram::{DiagramError, DiagramResult};
use crate::service::schema::SchemaError;
use axum::Json;
use dal::WsEvent;
use dal::{
    generate_name, node_position::NodePositionView, Component, DiagramKind, NodePosition,
    NodeTemplate, NodeView, Schema, SchemaId, StandardModel, SystemId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub schema_id: SchemaId,
    pub system_id: Option<SystemId>,
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

    let name = generate_name();
    let schema = Schema::get_by_id(&ctx, &request.schema_id)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let diagram_kind = schema
        .diagram_kind()
        .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
    if diagram_kind != DiagramKind::Configuration {
        return Err(DiagramError::InvalidDiagramKind(diagram_kind));
    }

    let (component, node) =
        Component::new_for_schema_variant_with_node(&ctx, &name, schema_variant_id).await?;

    let component_id = *component.id();

    let node_template = NodeTemplate::new_from_schema_id(&ctx, request.schema_id).await?;

    let position = NodePosition::new(
        &ctx,
        *node.id(),
        diagram_kind,
        request.system_id,
        request.x.clone(),
        request.y.clone(),
    )
    .await?;
    let positions = vec![NodePositionView::from(position)];
    let node_view = NodeView::new(name, &node, component_id, positions, node_template);

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(CreateNodeResponse { node: node_view }))
}
