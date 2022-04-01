use super::TestResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};
use axum::Json;
use dal::billing_account::BillingAccountSignup;
use dal::test_harness::generate_fake_name;
use dal::{BillingAccount, HistoryActor, ReadTenancy, WriteTenancy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: BillingAccountSignup,
}

pub async fn signup(
    HandlerContext(builder, mut txns): HandlerContext,
    JwtSecretKey(_jwt_secret_key): JwtSecretKey,
) -> TestResult<Json<SignupResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(
        dal::context::AccessBuilder::new(
            ReadTenancy::new_universal(),
            WriteTenancy::new_universal(),
            HistoryActor::SystemInit,
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

    txns.commit().await?;
    Ok(Json(SignupResponse { data: result }))
}
