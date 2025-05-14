use axum::Json;
use dal::{
    ChangeSet,
    Secret,
    SecretAlgorithm,
    SecretId,
    SecretVersion,
    SecretView,
    WsEvent,
    key_pair::KeyPairPk,
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    HandlerContext,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;

use crate::SecretResult;

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
) -> SecretResult<ForceChangeSetResponse<SecretView>> {
    let mut ctx = builder.build(request_tx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Update secret metadata.
    let mut secret = Secret::get_by_id(&ctx, request.id).await?;
    secret = secret
        .update_metadata(&ctx, request.name, request.description)
        .await?;

    // Update encrypted contents.
    if let Some(new_data) = request.new_secret_data {
        secret = secret
            .update_encrypted_contents(
                &ctx,
                new_data.crypted.as_slice(),
                new_data.key_pair_pk,
                new_data.version,
                new_data.algorithm,
            )
            .await?;
    }

    ctx.write_audit_log(
        AuditLogKind::UpdateSecret {
            name: secret.name().to_string(),
            secret_id: secret.id(),
        },
        secret.name().to_string(),
    )
    .await?;

    WsEvent::secret_updated(&ctx, secret.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        SecretView::from_secret(&ctx, secret, None).await?,
    ))
}
