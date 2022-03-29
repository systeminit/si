use axum::extract::Query;
use axum::Json;
use dal::{secret::SecretView, Secret, StandardModel, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SecretResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretResponse {
    pub list: Vec<SecretView>,
}

pub async fn list_secrets(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListSecretRequest>,
) -> SecretResult<Json<ListSecretResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let list: Vec<SecretView> = Secret::list(&ctx)
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    let response = ListSecretResponse { list };

    Ok(Json(response))
}
