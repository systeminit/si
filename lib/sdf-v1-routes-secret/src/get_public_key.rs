use axum::Json;
use dal::PublicKey;
use sdf_extract::{
    HandlerContext,
    v1::AccessBuilder,
};

use crate::SecretResult;

pub type GetPublicKeyResponse = PublicKey;

pub async fn get_public_key(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> SecretResult<Json<GetPublicKeyResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let response: GetPublicKeyResponse = PublicKey::get_current(&ctx).await?;

    Ok(Json(response))
}
