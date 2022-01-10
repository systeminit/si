use axum::{extract::Query, Json};
use dal::{node::NodeId, schematic::Schematic, system::SystemId, Tenancy, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchematicRequest {
    pub root_node_id: NodeId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSchematicResponse = Schematic;

pub async fn get_schematic(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
    Query(request): Query<GetSchematicRequest>,
) -> SchematicResult<Json<GetSchematicResponse>> {
    let txn = txn.start().await?;
    let mut tenancy = Tenancy::new_workspace(vec![request.workspace_id]);
    tenancy.billing_account_ids = vec![claim.billing_account_id];
    tenancy.universal = true;

    let response = Schematic::find(
        &txn,
        &tenancy,
        &request.visibility,
        request.system_id,
        request.root_node_id,
    )
    .await?;
    Ok(Json(response))
}
