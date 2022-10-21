use axum::Json;
use dal::{
    billing_account::BillingAccountSignup, BillingAccount, HistoryActor, RequestContext,
    StandardModel,
};
use serde::{Deserialize, Serialize};

use super::{generate_fake_name, TestResult};
use crate::server::extract::{HandlerContext, JwtSecretKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignupResponse {
    data: BillingAccountSignup,
    jwt: String,
}

pub async fn signup_and_login(
    HandlerContext(builder): HandlerContext,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
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
    let jwt = result
        .user
        .login(
            &ctx,
            &jwt_secret_key,
            result.billing_account.id(),
            user_password,
        )
        .await?;

    ctx.commit().await?;

    Ok(Json(SignupResponse { data: result, jwt }))
}
