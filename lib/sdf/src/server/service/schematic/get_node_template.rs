use axum::extract::Query;
use axum::Json;
use dal::node::NodeTemplate;
use dal::{SchemaId, Tenancy, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
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
    let mut tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    tenancy.workspace_ids = vec![request.workspace_id];
    tenancy.universal = true;

    let response =
        NodeTemplate::new_from_schema_id(&txn, &tenancy, &request.visibility, request.schema_id)
            .await?;

    Ok(Json(response))
}
