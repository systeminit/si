use std::collections::HashMap;

use axum::{
    Json,
    extract::Path,
};
use dal::{
    PublicKey,
    Secret,
    SecretAlgorithm,
    SecretId,
    SecretVersion,
    WsEvent,
};
use serde::{
    Deserialize,
    Serialize,
};
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
    encrypt_message,
};
use crate::{
    api_types::secrets::v1::SecretV1,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/secrets/{secret_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("secret_id" = String, Path, description = "Secret identifier")
    ),
    tag = "secrets",
    summary = "Update a secret",
    request_body = UpdateSecretV1Request,
    responses(
        (status = 200, description = "Secret updated successfully", body = UpdateSecretV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Secret not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn update_secret(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SecretV1RequestPath { secret_id }): Path<SecretV1RequestPath>,
    payload: Result<Json<UpdateSecretV1Request>, axum::extract::rejection::JsonRejection>,
) -> SecretsResult<Json<UpdateSecretV1Response>> {
    let Json(payload) = payload?;

    let secret = Secret::get_by_id(ctx, secret_id)
        .await
        .map_err(|_s| SecretsError::SecretNotFound(secret_id))?;

    let secret = secret
        .update_metadata(ctx, payload.name, payload.description)
        .await?;

    let public_key = PublicKey::get_current(ctx).await?;
    let algorithm = SecretAlgorithm::Sealedbox;
    let version = SecretVersion::V1;

    let secret = if let Some(updated_data) = payload.raw_data {
        let encrypted_message = encrypt_message(updated_data, &public_key).await;
        secret
            .update_encrypted_contents(
                ctx,
                &encrypted_message,
                *public_key.pk(),
                version,
                algorithm,
            )
            .await?
    } else {
        secret
    };

    ctx.write_audit_log(
        AuditLogKind::UpdateSecret {
            name: secret.name().to_string(),
            secret_id: secret.id(),
        },
        secret.name().to_string(),
    )
    .await?;

    WsEvent::secret_updated(ctx, secret.id())
        .await?
        .publish_on_commit(ctx)
        .await?;

    tracker.track(
        ctx,
        "api_update_secret",
        json!({
            "secret_name": secret.name().to_string(),
            "secret_definition": secret.definition().to_string()
        }),
    );

    ctx.commit().await?;

    Ok(Json(UpdateSecretV1Response {
        secret: SecretV1 {
            id: secret_id,
            name: secret.name().to_string(),
            definition: secret.definition().to_string(),
            description: secret.description().clone(),
        },
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecretV1Request {
    #[schema(value_type = String, example = "01HAXYZF3GC9CYA6ZVSM3E4YHH")]
    pub id: SecretId,
    #[schema(value_type = String, example = "AWS Access Key")]
    pub name: String,
    #[schema(value_type = String, example = "Updated AWS Secret Key for EC2 access")]
    pub description: Option<String>,
    #[schema(value_type = HashMap<String, String>, example = json!({
        "access_key_id": "AKIAIOSFODNN7EXAMPLE",
        "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        "region": "us-west-2",
        "default_output": "json"
    }))]
    #[serde(default)]
    pub raw_data: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSecretV1Response {
    #[schema(example = json!({
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
        "name": "Updated AWS Secret Key",
        "definition": "aws_secret_key",
        "description": "Updated AWS Secret Key for EC2 access"
    }))]
    pub secret: SecretV1,
}
