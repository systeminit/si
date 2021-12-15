use axum::Json;
use dal::billing_account::BillingAccountSignup;
use dal::{
    test_harness::generate_fake_name, BillingAccount, HistoryActor, StandardModel, Tenancy,
    Visibility,
};
use serde::{Deserialize, Serialize};

use super::TestResult;
use crate::server::extract::{JwtSecretKey, NatsTxn, PgRwTxn};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: BillingAccountSignup,
    jwt: String,
}

pub async fn signup_and_login(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
) -> TestResult<Json<SignupResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let billing_account_name = generate_fake_name();
    let user_name = generate_fake_name();
    let user_email = format!("{}@example.com", user_name);
    let user_password = "snakes";

    let result = BillingAccount::signup(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await?;
    let jwt = result
        .user
        .login(
            &txn,
            &jwt_secret_key,
            result.billing_account.id(),
            user_password,
        )
        .await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(SignupResponse { data: result, jwt }))
}
