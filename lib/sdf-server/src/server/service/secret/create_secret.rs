use axum::Json;
use dal::secret::SecretView;
use dal::{
    key_pair::KeyPairPk, EncryptedSecret, SecretAlgorithm, SecretVersion, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::SecretResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    pub name: String,
    pub definition: String,
    pub description: Option<String>,
    pub crypted: Vec<u8>,
    pub key_pair_pk: KeyPairPk,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type CreateSecretResponse = SecretView;

pub async fn create_secret(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_tx): AccessBuilder,
    Json(request): Json<CreateSecretRequest>,
) -> SecretResult<Json<CreateSecretResponse>> {
    let ctx = builder.build(request_tx.build(request.visibility)).await?;

    let secret = EncryptedSecret::new(
        &ctx,
        request.name,
        request.definition,
        request.description,
        &request.crypted,
        request.key_pair_pk,
        request.version,
        request.algorithm,
    )
    .await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(SecretView::from_secret(&ctx, secret).await?))
}
