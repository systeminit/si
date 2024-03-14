use axum::response::IntoResponse;
use axum::Json;
use dal::secret::SecretView;
use dal::{
    key_pair::KeyPairPk, ChangeSetPointer, EncryptedSecret, Secret, SecretAlgorithm, SecretVersion,
    Visibility, WsEvent,
};
use dal::{HistoryActor, SecretError, SecretId, StandardModel};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::SecretResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewSecretData {
    pub crypted: Vec<u8>,
    pub key_pair_pk: KeyPairPk,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecretRequest {
    pub id: SecretId,
    pub name: String,
    pub description: Option<String>,
    pub new_secret_data: Option<NewSecretData>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type UpdateSecretResponse = SecretView;

pub async fn update_secret(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_tx): AccessBuilder,
    Json(request): Json<UpdateSecretRequest>,
) -> SecretResult<impl IntoResponse> {
    let mut ctx = builder.build(request_tx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSetPointer::force_new(&mut ctx).await?;

    let mut secret = EncryptedSecret::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SecretError::SecretNotFound(request.id))?;

    // UPDATE SECRET METADATA
    secret.set_name(&ctx, request.name).await?;
    secret.set_description(&ctx, request.description).await?;
    match ctx.history_actor() {
        HistoryActor::SystemInit => {}
        HistoryActor::User(id) => {
            println!("before - {id}");
            secret.set_updated_by(&ctx, Some(*id)).await?;
            println!("done - {id}");
        }
    }

    // UPDATE SECRET ECRYPTED CONTENTS
    if let Some(new_data) = request.new_secret_data {
        secret.set_crypted(&ctx, new_data.crypted).await?;
        secret.set_key_pair_pk(&ctx, new_data.key_pair_pk).await?;
        secret.set_version(&ctx, new_data.version).await?;
        secret.set_algorithm(&ctx, new_data.algorithm).await?;
    }

    // TODO(nick): unify this with the encrypted secrets stuff. For now, let's update the referential secret
    // as a side effect.
    Secret::update(&ctx, &secret).await?;

    WsEvent::secret_updated(&ctx, *secret.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    Ok(response.body(serde_json::to_string(
        &SecretView::from_secret(&ctx, secret.into()).await?,
    )?)?)
}
