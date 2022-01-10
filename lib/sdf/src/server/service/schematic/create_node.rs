use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use crate::service::schematic::{SchematicError, SchematicResult};
use axum::Json;
use dal::{
    generate_name, Component, HistoryActor, NodeTemplate, NodeView, SchemaId, StandardModel,
    Tenancy, Visibility, Workspace, WorkspaceId,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub schema_id: SchemaId,
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
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let name = generate_name(None);

    let (component, node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        &tenancy,
        &request.visibility,
        &history_actor,
        &name,
        &request.schema_id,
    )
    .await?;

    let mut schema_tenancy = tenancy.clone();
    schema_tenancy.universal = true;

    let node_template = NodeTemplate::new_from_schema_id(
        &txn,
        &schema_tenancy,
        &request.visibility,
        request.schema_id,
    )
    .await?;

    // TODO: this creates a node without a position, it will be upserted when dragged, but it's a
    // problem. We need to pass the position in CreateNodeRequest
    let node_view = NodeView::new(component.name(), node, vec![], node_template);

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(CreateNodeResponse { node: node_view }))
}
