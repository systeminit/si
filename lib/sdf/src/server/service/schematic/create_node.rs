use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::schematic::{SchematicError, SchematicResult};
use axum::Json;
use dal::{
    generate_name, node::NodeId, Component, Node, NodeKind, NodePosition, NodeTemplate, NodeView,
    Schema, SchemaId, SchematicKind, StandardModel, SystemId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub schema_id: SchemaId,
    pub root_node_id: NodeId,
    pub system_id: Option<SystemId>,
    pub x: String,
    pub y: String,
    pub parent_node_id: Option<NodeId>,
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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateNodeRequest>,
) -> SchematicResult<Json<CreateNodeResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let name = generate_name(None);
    let schema = Schema::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &request.schema_id,
    )
    .await?
    .ok_or(SchematicError::SchemaNotFound)?;

    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(SchematicError::SchemaVariantNotFound)?;

    let system_id = match &request.system_id {
        Some(system_id) => system_id,
        None => return Err(SchematicError::InvalidSystem),
    };
    let (_component, node) = match (SchematicKind::from(*schema.kind()), &request.parent_node_id) {
        (SchematicKind::Component, Some(parent_node_id)) => {
            let parent_node = Node::get_by_id(
                ctx.pg_txn(),
                &ctx.read_tenancy().into(),
                ctx.visibility(),
                parent_node_id,
            )
            .await?;
            // Ensures parent node must be a NodeKind::Deployment
            if let Some(parent_node) = parent_node {
                match parent_node.kind() {
                    NodeKind::Component | NodeKind::System => {
                        return Err(SchematicError::InvalidParentNode(*parent_node.kind()))
                    }
                    NodeKind::Deployment => {}
                }
            } else {
                return Err(SchematicError::ParentNodeNotFound(*parent_node_id));
            }
            Component::new_for_schema_variant_with_node_in_deployment(
                ctx.pg_txn(),
                ctx.nats_txn(),
                ctx.veritech().clone(),
                ctx.encryption_key(),
                &ctx.write_tenancy().into(),
                ctx.visibility(),
                ctx.history_actor(),
                &name,
                schema_variant_id,
                system_id,
                parent_node_id,
            )
            .await?
        }
        (SchematicKind::Deployment, None) => {
            Component::new_for_schema_variant_with_node_in_system(
                ctx.pg_txn(),
                ctx.nats_txn(),
                ctx.veritech().clone(),
                ctx.encryption_key(),
                &ctx.write_tenancy().into(),
                ctx.visibility(),
                ctx.history_actor(),
                &name,
                schema_variant_id,
                system_id,
            )
            .await?
        }
        (schema_kind, parent_node_id) => {
            return Err(SchematicError::InvalidSchematicKindParentNodeIdPair(
                schema_kind,
                *parent_node_id,
            ))
        }
    };

    let node_template = NodeTemplate::new_from_schema_id(
        ctx.pg_txn(),
        ctx.read_tenancy(),
        ctx.visibility(),
        request.schema_id,
    )
    .await?;

    let mut position = NodePosition::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        (*node.kind()).into(),
        request.root_node_id,
        request.x,
        request.y,
    )
    .await?;
    if let Some(system_id) = request.system_id {
        position
            .set_system_id(
                ctx.pg_txn(),
                ctx.nats_txn(),
                ctx.visibility(),
                ctx.history_actor(),
                Some(system_id),
            )
            .await?;
    }
    position
        .set_node(
            ctx.pg_txn(),
            ctx.nats_txn(),
            ctx.visibility(),
            ctx.history_actor(),
            node.id(),
        )
        .await?;
    let node_view = NodeView::new(name, node, vec![position], node_template);

    txns.commit().await?;
    Ok(Json(CreateNodeResponse { node: node_view }))
}
