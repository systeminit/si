use anyhow::Result;
use axum::Json;
use dal::PublicKey;

use crate::extract::{v1::AccessBuilder, HandlerContext};

pub type GetPublicKeyResponse = PublicKey;

pub async fn get_public_key(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> Result<Json<GetPublicKeyResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let response: GetPublicKeyResponse = PublicKey::get_current(&ctx).await?;

    Ok(Json(response))
}
