use axum::Json;
use dal::PublicKey;

use super::SecretResult;
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

pub type GetPublicKeyResponse = PublicKey;

pub async fn get_public_key(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
) -> SecretResult<Json<GetPublicKeyResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let response: GetPublicKeyResponse = PublicKey::get_current(
        ctx.pg_txn(),
        ctx.read_tenancy(),
        ctx.visibility(),
        &claim.billing_account_id,
    )
    .await?;

    txns.commit().await?;
    Ok(Json(response))
}
