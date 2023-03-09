use super::SessionError;
use super::SessionResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};
use axum::Json;
use dal::{context::AccessBuilder, BillingAccount, HistoryActor, Tenancy, User};
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
    HandlerContext(builder): HandlerContext,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
    Json(request): Json<LoginRequest>,
) -> SessionResult<Json<LoginResponse>> {
    // Global history actor
    let mut ctx = builder
        .build(
            AccessBuilder::new(
                // Empty tenancy means things can be written, but won't ever be read by whatever uses the standard model
                Tenancy::new_empty(),
                HistoryActor::SystemInit,
            )
            .build_head(),
        )
        .await?;
    let billing_account = BillingAccount::find_by_name(&ctx, &request.billing_account_name)
        .await?
        .ok_or(SessionError::LoginFailed)?;
    let billing_account_defaults = BillingAccount::get_defaults(&ctx, billing_account.pk()).await?;

    ctx.update_tenancy(Tenancy::new(*billing_account_defaults.workspace.pk()));

    let user = User::find_by_email(&ctx, &request.user_email)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    // Update context history actor
    ctx.update_history_actor(HistoryActor::User(user.pk()));

    let jwt = user
        .login(
            &ctx,
            &jwt_secret_key,
            billing_account_defaults.workspace.pk(),
            &request.user_password,
        )
        .await
        .map_err(|_| SessionError::LoginFailed)?;

    Ok(Json(LoginResponse {
        jwt,
        user,
        billing_account,
    }))
}
