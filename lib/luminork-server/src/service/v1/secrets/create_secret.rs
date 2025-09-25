use std::collections::HashMap;

use axum::Json;
use dal::{
    PublicKey,
    Secret,
    SecretAlgorithm,
    SecretVersion,
    WsEvent,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::ToSchema;

use super::{
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
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/secrets",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "secrets",
    summary = "Create a secret",
    request_body = CreateSecretV1Request,
    responses(
        (status = 200, description = "Secret created successfully", body = CreateSecretV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_secret(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateSecretV1Request>, axum::extract::rejection::JsonRejection>,
) -> SecretsResult<Json<CreateSecretV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SecretsError::NotPermittedOnHead);
    }

    let public_key = PublicKey::get_current(ctx).await?;
    let algorithm = SecretAlgorithm::Sealedbox;
    let version = SecretVersion::V1;

    let encrypted_message = encrypt_message(payload.raw_data, &public_key);

    let secret = Secret::new(
        ctx,
        payload.name,
        payload.definition_name,
        payload.description,
        &encrypted_message.await,
        *public_key.pk(),
        version,
        algorithm,
    )
    .await?;

    tracker.track(
        ctx,
        "api_create_secret",
        json!({
            "secret_name": secret.name().to_string(),
            "secret_definition": secret.definition().to_string()
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::CreateSecret {
            name: secret.name().to_string(),
            secret_id: secret.id(),
        },
        secret.name().to_string(),
    )
    .await?;

    WsEvent::secret_created(ctx, secret.id())
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(CreateSecretV1Response {
        secret: SecretV1 {
            id: secret.id(),
            name: secret.name().to_string(),
            definition: secret.definition().to_string(),
            description: secret.description().clone(),
        },
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretV1Request {
    #[schema(value_type = String, example = "AWS Access Key")]
    pub name: String,
    #[schema(value_type = String, example = "aws_credentials")]
    pub definition_name: String,
    #[schema(value_type = Option<String>, example = "AWS credentials for production environment")]
    pub description: Option<String>,
    #[schema(value_type = HashMap<String, String>, example = json!({
        "access_key_id": "AKIAIOSFODNN7EXAMPLE",
        "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
        "region": "us-west-2",
        "default_output": "json"
    }))]
    #[serde(default)]
    pub raw_data: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretV1Response {
    #[schema(example = json!({
        "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
        "name": "AWS Access Key",
        "definition": "aws_credentials",
        "description": "AWS credentials for production environment"
    }))]
    pub secret: SecretV1,
}
