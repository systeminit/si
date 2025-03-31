use axum::Json;
use dal::{ChangeSet, Secret, SecretId, SecretView, Visibility, WsEvent};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use super::{SecretError, SecretResult};
use crate::extract::{v1::AccessBuilder, HandlerContext};
use crate::service::force_change_set_response::ForceChangeSetResponse;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSecretRequest {
    pub id: SecretId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type UpdateSecretResponse = SecretView;

pub async fn delete_secret(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_tx): AccessBuilder,
    Json(request): Json<DeleteSecretRequest>,
) -> SecretResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_tx.build(request.visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Delete Secret
    let secret = Secret::get_by_id(&ctx, request.id).await?;

    let connected_components = secret.clone().find_connected_components(&ctx, None).await?;
    if !connected_components.is_empty() {
        return Err(SecretError::CantDeleteSecret(request.id));
    }

    ctx.write_audit_log(
        AuditLogKind::DeleteSecret {
            name: secret.name().to_string(),
            secret_id: secret.id(),
        },
        secret.name().to_string(),
    )
    .await?;

    secret.delete(&ctx).await?;

    WsEvent::secret_deleted(&ctx, request.id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
