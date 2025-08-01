use std::collections::HashMap;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    Component,
    ComponentId,
    Prop,
    PropId,
    Secret,
    SecretId,
    WsEvent,
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;

use super::{
    ComponentError,
    ComponentResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePropertyEditorValueRequest {
    pub attribute_value_id: AttributeValueId,
    pub parent_attribute_value_id: Option<AttributeValueId>,
    pub prop_id: PropId,
    pub component_id: ComponentId,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
    pub is_for_secret: bool,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn update_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<UpdatePropertyEditorValueRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Cache the "before value" before updating for audit logging.
    let before_value = AttributeValue::get_by_id(&ctx, request.attribute_value_id)
        .await?
        .value(&ctx)
        .await?;

    // Determine how to update the value based on whether it corresponds to a secret. The vast
    // majority of the time, the request will not be for a secret.
    if request.is_for_secret {
        if let Some(value) = request.value.as_ref() {
            let secret_id: SecretId = serde_json::from_value(value.to_owned())?;
            Secret::attach_for_attribute_value(&ctx, request.attribute_value_id, Some(secret_id))
                .await?;
        } else {
            Secret::attach_for_attribute_value(&ctx, request.attribute_value_id, None).await?;
        }
    } else {
        AttributeValue::update(&ctx, request.attribute_value_id, request.value.to_owned()).await?;

        if request.value != before_value {
            // If the values have changed then we should enqueue an update action
            // if the values haven't changed then we can skip this update action as it is usually a no-op
            Component::enqueue_update_action_if_applicable(&ctx, request.attribute_value_id)
                .await?;
        }
    }

    let component = Component::get_by_id(&ctx, request.component_id).await?;

    {
        let component_schema = component.schema(&ctx).await?;
        let component_schema_variant = component.schema_variant(&ctx).await?;
        let prop = Prop::get_by_id(&ctx, request.prop_id).await?;

        // Determine the value after updating for audit logging.
        if request.is_for_secret {
            let (before_secret_id, before_secret_name) = if let Some(inner) = before_value {
                let secret_key = Secret::key_from_value_in_attribute_value(inner.to_owned())?;
                let secret_id = Secret::get_id_by_key_or_error(&ctx, secret_key).await?;
                let secret_name = Secret::get_by_id(&ctx, secret_id).await?.name().to_string();
                (Some(secret_id), Some(secret_name))
            } else {
                (None, None)
            };

            let (after_secret_id, after_secret_name) = if let Some(inner) = request.value {
                let secret_id: SecretId = serde_json::from_value(inner)
                    .map_err(ComponentError::SecretIdDeserialization)?;
                let secret_name = Secret::get_by_id(&ctx, secret_id).await?.name().to_string();
                (Some(secret_id), Some(secret_name))
            } else {
                (None, None)
            };

            ctx.write_audit_log(
                AuditLogKind::UpdatePropertyEditorValueForSecret {
                    component_id: request.component_id,
                    component_name: component.name(&ctx).await?,
                    schema_variant_id: component_schema_variant.id(),
                    schema_variant_display_name: component_schema_variant
                        .display_name()
                        .to_string(),
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
        } else {
            ctx.write_audit_log(
                AuditLogKind::UpdatePropertyEditorValue {
                    component_id: request.component_id,
                    component_name: component.name(&ctx).await?,
                    schema_variant_id: component_schema_variant.id(),
                    schema_variant_display_name: component_schema_variant
                        .display_name()
                        .to_string(),
                    prop_id: prop.id,
                    prop_name: prop.name.to_owned(),
                    attribute_value_id: request.attribute_value_id,
                    attribute_path: format!("/domain/{}", prop.name), // Fallback path for v1 route
                    before_value,
                    after_value: request.value,
                },
                prop.name.to_owned(),
            )
            .await?;
        }

        let parent_prop_id = match request.parent_attribute_value_id {
            Some(av_id) => AttributeValue::prop_id_opt(&ctx, av_id).await?,
            None => None,
        };
        let parent_prop_name = match parent_prop_id {
            Some(parent_prop_id) => Some(Prop::node_weight(&ctx, parent_prop_id).await?.name),
            None => None,
        };

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "property_value_updated",
            serde_json::json!({
                "how": "/component/property_value_updated",
                "component_id": component.id(),
                "component_schema_name": component_schema.name(),
                "prop_id": prop.id,
                "prop_name": prop.name,
                "parent_prop_id": parent_prop_id,
                "parent_prop_name": parent_prop_name,
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(
            &ctx,
            None,
            component.change_status(&ctx).await?,
            &mut socket_map,
        )
        .await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
