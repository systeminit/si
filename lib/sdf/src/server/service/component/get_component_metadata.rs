use axum::extract::Query;
use axum::Json;
use dal::{Component, ComponentId, StandardModel, Tenancy, Visibility, Workspace, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentMetadataRequest {
    pub component_id: ComponentId,
    //pub system_id: SystemId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentMetadataResponse {
    pub schema_name: String,
    //pub resource_state: Option<()>
}

pub async fn get_component_metadata(
    mut txn: PgRoTxn,
    Query(request): Query<GetComponentMetadataRequest>,
    Authorization(claim): Authorization,
) -> ComponentResult<Json<GetComponentMetadataResponse>> {
    let txn = txn.start().await?;
    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ComponentError::InvalidRequest)?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let component =
        Component::get_by_id(&txn, &tenancy, &request.visibility, &request.component_id)
            .await?
            .ok_or(ComponentError::NotFound)?;
    let mut schema_tenancy = tenancy.clone();
    schema_tenancy.universal = true;

    let schema = component
        .schema_with_tenancy(&txn, &schema_tenancy, &request.visibility)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?;
    let response = GetComponentMetadataResponse {
        schema_name: schema.name().to_owned(),
    };
    Ok(Json(response))
}
