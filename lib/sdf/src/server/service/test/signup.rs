use axum::Json;
use dal::{
    billing_account::BillingAccountSignup, test_harness::generate_fake_name, BillingAccount,
    HistoryActor, RequestContext,
};
use serde::{Deserialize, Serialize};

use super::TestResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: BillingAccountSignup,
}

pub async fn signup(
    HandlerContext(builder): HandlerContext,
    JwtSecretKey(_jwt_secret_key): JwtSecretKey,
) -> TestResult<Json<SignupResponse>> {
    let ctx = builder
        .build(RequestContext::new_universal_head(HistoryActor::SystemInit))
        .await?;

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

    ctx.commit().await?;

    Ok(Json(SignupResponse { data: result }))
}
