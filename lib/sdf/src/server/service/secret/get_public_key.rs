use axum::Json;
use dal::{PublicKey, Tenancy, Visibility};

use super::SecretResult;
use crate::server::extract::{Authorization, PgRoTxn};

pub type GetPublicKeyResponse = PublicKey;

pub async fn get_public_key(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
) -> SecretResult<Json<GetPublicKeyResponse>> {
    let txn = txn.start().await?;
    let mut tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let response: GetPublicKeyResponse =
        PublicKey::get_current(&txn, &tenancy, &visibility, &claim.billing_account_id).await?;
    Ok(Json(response))
}
