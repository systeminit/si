use axum::Json;
use dal::{BillingAccount, StandardModel, User};
use serde::{Deserialize, Serialize};

use super::{SessionError, SessionResult};
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub billing_account: BillingAccount,
}

pub async fn restore_authentication(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    // Why is this here?
    let billing_account = BillingAccount::get_by_pk(
        &ctx,
        &claim.find_billing_account_pk_for_workspace(&ctx).await?,
    )
    .await?;

    let user = User::get_by_id(&ctx, &claim.user_id)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    let reply = RestoreAuthenticationResponse {
        user,
        billing_account,
    };

    Ok(Json(reply))
}
