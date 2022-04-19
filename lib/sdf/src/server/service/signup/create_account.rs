use super::SignupResult;
use crate::{
    server::extract::{HandlerContext, SignupSecret},
    service::signup::SignupError,
};
use axum::Json;
use dal::{BillingAccount, HistoryActor, ReadTenancy, WriteTenancy};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub billing_account_name: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
    pub signup_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountResponse {
    pub success: bool,
}

pub async fn create_account(
    HandlerContext(builder, mut txns): HandlerContext,
    SignupSecret(signup_secret): SignupSecret,
    Json(request): Json<CreateAccountRequest>,
) -> SignupResult<Json<CreateAccountResponse>> {
    if signup_secret.as_str() != request.signup_secret.as_str() {
        warn!("invalid signup secret provided when signing up new billing account");
        return Err(SignupError::InvalidSignupSecret);
    }

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

    let _result = BillingAccount::signup(
        &ctx,
        &request.billing_account_name,
        &request.user_name,
        &request.user_email,
        &request.user_password,
    )
    .await?;

    txns.commit().await?;
    Ok(Json(CreateAccountResponse { success: true }))
}
