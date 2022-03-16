use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use crate::service::schematic::{SchematicError, SchematicResult};
use axum::Json;
use dal::node::NodeId;
use dal::{
    HistoryActor, NodePosition, SchematicKind, StandardModel, SystemId, Tenancy, Visibility,
    Workspace, WorkspaceId, WriteTenancy,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
    pub node_id: NodeId,
    pub schematic_kind: SchematicKind,
    pub root_node_id: NodeId,
    pub system_id: Option<SystemId>,
    pub x: String,
    pub y: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNodePositionResponse {
    pub position: NodePosition,
}

pub async fn set_node_position(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<SetNodePositionRequest>,
) -> SchematicResult<Json<SetNodePositionResponse>> {
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

    let write_tenancy = WriteTenancy::new_workspace(*workspace.id());
    let position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &write_tenancy,
        &request.visibility,
        &history_actor,
        request.schematic_kind,
        &request.system_id,
        request.root_node_id,
        request.node_id,
        &request.x,
        &request.y,
    )
    .await?;

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(SetNodePositionResponse { position }))
}
