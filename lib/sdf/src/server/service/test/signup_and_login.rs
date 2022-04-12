use axum::Json;
use dal::billing_account::BillingAccountSignup;
use dal::{
    test_harness::generate_fake_name, BillingAccount, HistoryActor, ReadTenancy, StandardModel,
    WriteTenancy,
};
use serde::{Deserialize, Serialize};

use super::TestResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: BillingAccountSignup,
    jwt: String,
}

pub async fn signup_and_login(
    HandlerContext(builder, mut txns): HandlerContext,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
) -> TestResult<Json<SignupResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(
        dal::context::AccessBuilder::new(
            ReadTenancy::new_universal(),
            WriteTenancy::new_universal(),
            HistoryActor::SystemInit,
            None,
        )
        .build_head(),
        &txns,
    );

    let billing_account_name = generate_fake_name();
    let user_name = generate_fake_name();
    let user_email = format!("{}@example.com", user_name);
    let user_password = "snakes";

    let result = BillingAccount::signup(
        &ctx,
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await?;
    let jwt = result
        .user
        .login(
            &ctx,
            &jwt_secret_key,
            result.billing_account.id(),
            user_password,
        )
        .await?;

    txns.commit().await?;
    Ok(Json(SignupResponse { data: result, jwt }))
}
