use axum::Json;
use dal::PublicKey;

use super::SecretResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

pub type GetPublicKeyResponse = PublicKey;

pub async fn get_public_key(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
) -> SecretResult<Json<GetPublicKeyResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    let response: GetPublicKeyResponse = PublicKey::get_current(&ctx).await?;

    Ok(Json(response))
}
