use axum::extract::Query;
use axum::Json;
use dal::{secret::SecretView, Secret, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

use super::SecretResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretResponse {
    pub list: Vec<SecretView>,
}

pub async fn list_secrets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSecretRequest>,
) -> SecretResult<Json<ListSecretResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let list: Vec<SecretView> = Secret::list(&ctx)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    let response = ListSecretResponse { list };

    Ok(Json(response))
}
