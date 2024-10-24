use std::collections::HashMap;

use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    AttributeValue, AttributeValueId, ChangeSet, Component, ComponentId, Prop, PropId, Secret,
    SecretId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
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
        AttributeValue::update(&ctx, request.attribute_value_id, request.value).await?;
    }

    // Track
    let component = Component::get_by_id(&ctx, request.component_id).await?;
    {
        let component_schema = component.schema(&ctx).await?;
        let prop = Prop::get_by_id(&ctx, request.prop_id).await?;

        // In this context, there will always be a parent attribute value id
        let parent_prop = if let Some(att_val_id) = request.parent_attribute_value_id {
            AttributeValue::prop_opt(&ctx, att_val_id).await?
        } else {
            None
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
                "parent_prop_id": parent_prop.as_ref().map(|prop| prop.id),
                "parent_prop_name": parent_prop.as_ref().map(|prop| prop.name.clone()),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(&ctx, component.change_status(&ctx).await?, &mut socket_map)
        .await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
