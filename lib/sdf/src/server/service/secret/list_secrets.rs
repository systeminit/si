use axum::extract::Query;
use axum::Json;
use dal::{
    secret::SecretKind, secret::SecretObjectType, secret::SecretView, Secret, StandardModel,
    Tenancy, Visibility,
};
use serde::{Deserialize, Serialize};

use super::SecretResult;
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
pub struct ListSecretRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ListSecretResponse = Vec<SecretView>;

pub async fn list_secrets(
    mut txn: PgRoTxn,
    Query(request): Query<ListSecretRequest>,
    Authorization(claim): Authorization,
) -> SecretResult<Json<ListSecretResponse>> {
    let txn = txn.start().await?;
    let mut tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    tenancy.universal = true;
    let mut response: Vec<SecretView> = Secret::list(&txn, &tenancy, &request.visibility)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    // TODO: remove this when we can create some secret, but as we can't this helps with debugging
    response.push(SecretView {
        id: 1.into(),
        name: "moro num pais tropical".to_owned(),
        kind: SecretKind::DockerHub,
        object_type: SecretObjectType::Credential,
    });
    Ok(Json(response))
}
