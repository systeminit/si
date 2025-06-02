use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Secret,
    WsEvent,
};
use serde::Serialize;
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    SecretV1RequestPath,
    SecretsError,
    SecretsResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    delete,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("secret_id" = String, Path, description = "Secret identifier")
    ),
    tag = "secrets",
    summary = "Delete a secret",
    responses(
        (status = 200, description = "Secret deleted successfully", body = DeleteSecretV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Secret not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn delete_secret(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SecretV1RequestPath { secret_id }): Path<SecretV1RequestPath>,
) -> SecretsResult<Json<DeleteSecretV1Response>> {
    let secret = Secret::get_by_id(ctx, secret_id)
        .await
        .map_err(|_s| SecretsError::SecretNotFound(secret_id))?;

    let connected_components = secret.clone().find_connected_components(ctx, None).await?;
    if !connected_components.is_empty() {
        return Err(SecretsError::CantDeleteSecret(secret_id));
    }

    ctx.write_audit_log(
        AuditLogKind::DeleteSecret {
            name: secret.name().to_string(),
            secret_id: secret.id(),
        },
        secret.name().to_string(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_delete_secret",
        json!({
            "secret_id": secret_id,
            "secret_definition": secret.definition()
        }),
    );

    secret.delete(ctx).await?;

    WsEvent::secret_deleted(ctx, secret_id)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(DeleteSecretV1Response { success: true }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSecretV1Response {
    #[schema(value_type = bool)]
    pub success: bool,
}
