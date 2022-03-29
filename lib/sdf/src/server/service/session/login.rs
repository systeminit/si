use super::SessionError;
use super::SessionResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};
use axum::Json;
use dal::{
    context::AccessBuilder, BillingAccount, HistoryActor, ReadTenancy, StandardModel, User,
    WriteTenancy,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub billing_account_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: User,
    pub billing_account: BillingAccount,
    pub jwt: String,
}

pub async fn login(
    HandlerContext(builder, mut txns): HandlerContext,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
    Json(request): Json<LoginRequest>,
) -> SessionResult<Json<LoginResponse>> {
    let txns = txns.start().await?;
    // Global history actor
    let ctx = builder.build(
        AccessBuilder::new(
            ReadTenancy::new_universal(),
            // Empty tenancy means things can be written, but won't ever be read
            WriteTenancy::new_empty(),
            HistoryActor::SystemInit,
        )
        .build_head(),
        &txns,
    );
    let billing_account = BillingAccount::find_by_name(&ctx, &request.billing_account_name)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    // Update context tenancies
    let ctx = builder.build(
        AccessBuilder::new(
            ReadTenancy::new_billing_account(vec![*billing_account.id()]),
            WriteTenancy::new_billing_account(*billing_account.id()),
            HistoryActor::SystemInit,
        )
        .build_head(),
        &txns,
    );

    let user = User::find_by_email(&ctx, &request.user_email)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    // Update context history actor
    let ctx = builder.build(
        AccessBuilder::new(
            ctx.read_tenancy().clone(),
            ctx.write_tenancy().clone(),
            HistoryActor::User(*user.id()),
        )
        .build_head(),
        &txns,
    );

    let jwt = user
        .login(
            &ctx,
            &jwt_secret_key,
            billing_account.id(),
            &request.user_password,
        )
        .await
        .map_err(|_| SessionError::LoginFailed)?;

    txns.commit().await?;

    Ok(Json(LoginResponse {
        jwt,
        user,
        billing_account,
    }))
}
