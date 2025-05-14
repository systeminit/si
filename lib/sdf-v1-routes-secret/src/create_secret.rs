use axum::Json;
use dal::{
    ChangeSet,
    Secret,
    SecretAlgorithm,
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
) -> SecretResult<ForceChangeSetResponse<SecretView>> {
    let mut ctx = builder.build(request_tx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let secret = Secret::new(
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

    ctx.write_audit_log(
        AuditLogKind::CreateSecret {
            name: secret.name().to_string(),
            secret_id: secret.id(),
        },
        secret.name().to_string(),
    )
    .await?;

    WsEvent::secret_created(&ctx, secret.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let secret = SecretView::from_secret(&ctx, secret, None).await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, secret))
}
