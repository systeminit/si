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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    // Why is this here?
    let billing_account = BillingAccount::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &claim.billing_account_id,
    )
    .await?
    .ok_or(SessionError::LoginFailed)?;

    let user = User::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &claim.user_id,
    )
    .await?
    .ok_or(SessionError::LoginFailed)?;

    let reply = RestoreAuthenticationResponse {
        user,
        billing_account,
    };

    Ok(Json(reply))
}
