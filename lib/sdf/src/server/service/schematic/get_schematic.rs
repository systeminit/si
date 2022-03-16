use axum::{extract::Query, Json};
use dal::{
    node::NodeId, schematic::Schematic, system::SystemId, ReadTenancy, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::{SchematicError, SchematicResult};
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
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![request.workspace_id]).await?;
    if !read_tenancy
        .billing_accounts()
        .contains(&claim.billing_account_id)
    {
        return Err(SchematicError::NotAuthorized);
    }

    let response = Schematic::find(
        &txn,
        &read_tenancy,
        &request.visibility,
        request.system_id,
        request.root_node_id,
    )
    .await?;
    Ok(Json(response))
}
