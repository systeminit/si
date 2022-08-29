use axum::Json;
use serde::{Deserialize, Serialize};

use dal::{BillingAccount, HistoryActor, ReadTenancy, StandardModel, WriteTenancy};
use telemetry::prelude::*;

use crate::{
    server::extract::{HandlerContext, SignupSecret},
    service::signup::SignupError,
};

use super::SignupResult;

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
    let mut ctx = builder.build(
        dal::context::AccessBuilder::new(
            ReadTenancy::new_universal(),
            WriteTenancy::new_universal(),
            HistoryActor::SystemInit,
        )
        .build_head(),
        &txns,
    );

    let billing_acct = BillingAccount::signup(
        &ctx,
        &request.billing_account_name,
        &request.user_name,
        &request.user_email,
        &request.user_password,
    )
    .await?;

    ctx.update_tenancies(
        ReadTenancy::new_workspace(
            txns.pg(),
            vec![*billing_acct.workspace.id()],
            ctx.visibility(),
        )
        .await?,
        WriteTenancy::new_workspace(*billing_acct.workspace.id()),
    );

    txns.commit().await?;
    Ok(Json(CreateAccountResponse { success: true }))
}
