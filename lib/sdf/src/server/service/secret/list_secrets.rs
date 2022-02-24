use axum::extract::Query;
use axum::Json;
use dal::{secret::SecretView, Secret, StandardModel, Tenancy, Visibility, Workspace, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{SecretError, SecretResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretResponse {
    pub list: Vec<SecretView>,
}

pub async fn list_secrets(
    mut txn: PgRoTxn,
    Query(request): Query<ListSecretRequest>,
    Authorization(claim): Authorization,
) -> SecretResult<Json<ListSecretResponse>> {
    let txn = txn.start().await?;
    let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let workspace = Workspace::get_by_id(
        &txn,
        &billing_account_tenancy,
        &request.visibility,
        &request.workspace_id,
    )
    .await?
    .ok_or(SecretError::WorkspaceNotFound(request.workspace_id))?;
    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let list: Vec<SecretView> = Secret::list(&txn, &tenancy, &request.visibility)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    let response = ListSecretResponse { list };

    Ok(Json(response))
}
