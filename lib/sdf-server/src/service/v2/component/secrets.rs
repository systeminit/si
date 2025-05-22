use axum::{
    Json,
    Router,
    extract::Path,
    routing::{
        get,
        post,
    },
};
use dal::{
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    Component,
    Prop,
    PropId,
    PublicKey,
    Secret,
    SecretAlgorithm,
    SecretId,
    SecretVersion,
    key_pair::KeyPairPk,
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;

use super::{
    ComponentIdFromPath,
    Result,
};
use crate::app_state::AppState;

pub type GetPublicKeyResponse = PublicKey;

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(new_secret))
        .route("/public_key", get(get_public_key))
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    pub name: String,
    pub attribute_value_id: AttributeValueId,
    pub prop_id: PropId,
    pub definition: String,
    pub description: Option<String>,
    pub crypted: Vec<u8>,
    pub key_pair_pk: KeyPairPk,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretResponse {
    pub id: SecretId,
}

async fn new_secret(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentIdFromPath { component_id }): Path<ComponentIdFromPath>,
    Json(request): Json<CreateSecretRequest>,
) -> Result<ForceChangeSetResponse<CreateSecretResponse>> {
    // only in use by the new UI, no WsEvents needed!
    let force_change_set_id = ChangeSet::force_new(ctx).await?;
    let component = Component::get_by_id(ctx, component_id).await?;

    let secret = Secret::new(
        ctx,
        request.name.clone(),
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

    let secret_id = secret.id();
    Secret::attach_for_attribute_value(ctx, request.attribute_value_id, Some(secret_id)).await?;

    let component_schema_variant = component.schema_variant(ctx).await?;
    let prop = Prop::get_by_id(ctx, request.prop_id).await?;

    let before_value = AttributeValue::get_by_id(ctx, request.attribute_value_id)
        .await?
        .value(ctx)
        .await?;

    let (before_secret_id, before_secret_name) = if let Some(inner) = before_value {
        let secret_key = Secret::key_from_value_in_attribute_value(inner.to_owned())?;
        let secret_id = Secret::get_id_by_key_or_error(ctx, secret_key).await?;
        let secret_name = Secret::get_by_id(ctx, secret_id).await?.name().to_string();
        (Some(secret_id), Some(secret_name))
    } else {
        (None, None)
    };

    let after_secret_id = Some(secret_id);
    let after_secret_name = Some(request.name);

    ctx.write_audit_log(
        AuditLogKind::UpdatePropertyEditorValueForSecret {
            component_id,
            component_name: component.name(ctx).await?,
            schema_variant_id: component_schema_variant.id(),
            schema_variant_display_name: component_schema_variant.display_name().to_string(),
            prop_id: prop.id,
            prop_name: prop.name.to_owned(),
            attribute_value_id: request.attribute_value_id,
            before_secret_name,
            before_secret_id,
            after_secret_name,
            after_secret_id,
        },
        prop.name.to_owned(),
    )
    .await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "component_new_secret",
        json!({
            "how": "/component/secret",
            "component_id": component_id,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        CreateSecretResponse { id: secret.id() },
    ))
}

pub async fn get_public_key(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
) -> Result<Json<GetPublicKeyResponse>> {
    let response: GetPublicKeyResponse = PublicKey::get_current(ctx).await?;

    Ok(Json(response))
}
