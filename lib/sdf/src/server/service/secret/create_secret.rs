use axum::Json;
use dal::{
    key_pair::KeyPairId, EncryptedSecret, Secret, SecretAlgorithm, SecretKind, SecretObjectType,
    SecretVersion, Visibility, WorkspaceId, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

use super::SecretResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub crypted: Vec<u8>,
    pub key_pair_id: KeyPairId,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretResponse {
    pub secret: Secret,
}

pub async fn create_secret(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_tx): AccessBuilder,
    Authorization(claim): Authorization,
    Json(request): Json<CreateSecretRequest>,
) -> SecretResult<Json<CreateSecretResponse>> {
    let ctx = builder.build(request_tx.build(request.visibility)).await?;

    let secret = EncryptedSecret::new(
        &ctx,
        request.name,
        request.object_type,
        request.kind,
        &request.crypted,
        request.key_pair_id,
        request.version,
        request.algorithm,
        claim.billing_account_pk,
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(CreateSecretResponse { secret }))
}
