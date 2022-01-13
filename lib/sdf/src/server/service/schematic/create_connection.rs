use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use crate::service::schematic::{SchematicError, SchematicResult};
use axum::Json;
use dal::node::NodeId;
use dal::socket::SocketId;
use dal::{Connection, HistoryActor, StandardModel, Tenancy, Visibility, Workspace, WorkspaceId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub head_node_id: NodeId,
    pub head_socket_id: SocketId,
    pub tail_node_id: NodeId,
    pub tail_socket_id: SocketId,
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
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<CreateConnectionRequest>,
) -> SchematicResult<Json<CreateConnectionResponse>> {
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

    let connection = Connection::new(
        &txn,
        &nats,
        &tenancy,
        &request.visibility,
        &history_actor,
        &request.head_node_id,
        &request.head_socket_id,
        &request.tail_node_id,
        &request.tail_socket_id,
    )
    .await?;

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(CreateConnectionResponse { connection }))
}
