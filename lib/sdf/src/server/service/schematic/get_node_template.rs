use axum::extract::Query;
use axum::Json;
use dal::node::NodeTemplate;
use dal::{ReadTenancy, SchemaId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{SchematicError, SchematicResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeTemplateRequest {
    pub schema_id: SchemaId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetNodeTemplateResponse = NodeTemplate;

pub async fn get_node_template(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
    Query(request): Query<GetNodeTemplateRequest>,
) -> SchematicResult<Json<GetNodeTemplateResponse>> {
    let txn = txn.start().await?;
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![request.workspace_id]).await?;
    if !read_tenancy
        .billing_accounts()
        .contains(&claim.billing_account_id)
    {
        return Err(SchematicError::NotAuthorized);
    }

    let response = NodeTemplate::new_from_schema_id(
        &txn,
        &read_tenancy,
        &request.visibility,
        request.schema_id,
    )
    .await?;

    Ok(Json(response))
}
