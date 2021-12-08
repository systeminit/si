use axum::extract::Query;
use axum::Json;
use dal::{Component, Schema, StandardModel, Tenancy, Visibility, Workspace, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{ApplicationError, ApplicationResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationItem {
    pub application: Component,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListApplicationResponse {
    pub list: Vec<ListApplicationItem>,
}

pub async fn list_applications(
    mut txn: PgRoTxn,
    Query(request): Query<ListApplicationRequest>,
    Authorization(claim): Authorization,
) -> ApplicationResult<Json<ListApplicationResponse>> {
    let txn = txn.start().await?;
    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(ApplicationError::InvalidRequest)?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let universal_tenancy = Tenancy::new_universal();
    let schemas = Schema::find_by_attr(
        &txn,
        &universal_tenancy,
        &request.visibility,
        "name",
        &"application".to_string(),
    )
    .await?;
    let schema = schemas.first().ok_or(ApplicationError::SchemaNotFound)?;
    let list: Vec<ListApplicationItem> = schema
        .components(&txn, &tenancy, &request.visibility)
        .await?
        .into_iter()
        .map(|application| ListApplicationItem { application })
        .collect();
    let response = ListApplicationResponse { list };
    Ok(Json(response))
}
