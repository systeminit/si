use axum::response::IntoResponse;
use axum::Json;
use dal::secret::SecretView;
use dal::{
    key_pair::KeyPairPk, ChangeSet, EncryptedSecret, SecretAlgorithm, SecretVersion, Visibility,
    WsEvent,
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
) -> SecretResult<impl IntoResponse> {
    let mut ctx = builder.build(request_tx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

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

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    let secret = SecretView::from_secret(&ctx, secret).await?;

    Ok(response.body(serde_json::to_string(&secret)?)?)
}
