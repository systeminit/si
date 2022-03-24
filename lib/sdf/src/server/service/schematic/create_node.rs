use crate::server::extract::{Authorization, EncryptionKey, NatsTxn, PgRwTxn, Veritech};
use crate::service::schematic::{SchematicError, SchematicResult};
use axum::Json;
use dal::{
    generate_name, node::NodeId, Component, HistoryActor, Node, NodeKind, NodePosition,
    NodeTemplate, NodeView, ReadTenancy, Schema, SchemaId, SchematicKind, StandardModel, SystemId,
    Tenancy, Visibility, Workspace, WorkspaceId, WriteTenancy,
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
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Veritech(veritech): Veritech,
    EncryptionKey(encryption_key): EncryptionKey,
    Authorization(claim): Authorization,
    Json(request): Json<CreateNodeRequest>,
) -> SchematicResult<Json<CreateNodeResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(SchematicError::InvalidRequest)?;

    let name = generate_name(None);

    let write_tenancy = WriteTenancy::new_workspace(*workspace.id());
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![*workspace.id()]).await?;

    let schema = Schema::get_by_id(
        &txn,
        &(&read_tenancy).into(),
        &request.visibility,
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
                &txn,
                &(&read_tenancy).into(),
                &request.visibility,
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
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &(&write_tenancy).into(),
                &request.visibility,
                &history_actor,
                &name,
                schema_variant_id,
                system_id,
                parent_node_id,
            )
            .await?
        }
        (SchematicKind::Deployment, None) => {
            Component::new_for_schema_variant_with_node_in_system(
                &txn,
                &nats,
                veritech,
                &encryption_key,
                &(&write_tenancy).into(),
                &request.visibility,
                &history_actor,
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
        &txn,
        &read_tenancy,
        &request.visibility,
        request.schema_id,
    )
    .await?;

    let mut position = NodePosition::new(
        &txn,
        &nats,
        &write_tenancy,
        &request.visibility,
        &history_actor,
        (*node.kind()).into(),
        request.root_node_id,
        request.x,
        request.y,
    )
    .await?;
    if let Some(system_id) = request.system_id {
        position
            .set_system_id(
                &txn,
                &nats,
                &request.visibility,
                &history_actor,
                Some(system_id),
            )
            .await?;
    }
    position
        .set_node(&txn, &nats, &request.visibility, &history_actor, node.id())
        .await?;
    let node_view = NodeView::new(name, node, vec![position], node_template);

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(CreateNodeResponse { node: node_view }))
}
