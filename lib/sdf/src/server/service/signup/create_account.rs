use super::SignupResult;
use crate::server::extract::{NatsTxn, PgRwTxn};
use axum::Json;
use dal::{BillingAccount, HistoryActor, Tenancy, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub billing_account_name: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountResponse {
    pub success: bool,
}

pub async fn create_account(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Json(request): Json<CreateAccountRequest>,
) -> SignupResult<Json<CreateAccountResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _result = BillingAccount::signup(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &request.billing_account_name,
        &request.user_name,
        &request.user_email,
        &request.user_password,
    )
    .await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(CreateAccountResponse { success: true }))
}
