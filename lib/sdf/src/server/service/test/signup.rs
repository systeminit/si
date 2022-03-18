use super::TestResult;
use crate::server::extract::{NatsTxn, PgRwTxn};
use axum::Json;
use dal::billing_account::BillingAccountSignup;
use dal::test_harness::generate_fake_name;
use dal::{BillingAccount, HistoryActor, Visibility, WriteTenancy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: BillingAccountSignup,
}

pub async fn signup(mut txn: PgRwTxn, mut nats: NatsTxn) -> TestResult<Json<SignupResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let write_tenancy = WriteTenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let billing_account_name = generate_fake_name();
    let user_name = generate_fake_name();
    let user_email = format!("{}@example.com", user_name);
    let user_password = "snakes";

    let result = BillingAccount::signup(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(SignupResponse { data: result }))
}
