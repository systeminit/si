use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::{
    AttributeValue, AttributeValueId, ChangeSet, Component, ComponentId, Prop, PropId, Secret,
    SecretId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePropertyEditorValueRequest {
    pub attribute_value_id: AttributeValueId,
    pub parent_attribute_value_id: Option<AttributeValueId>,
    pub prop_id: PropId,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
    pub is_for_secret: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertPropertyEditorValueRequest {
    pub parent_attribute_value_id: AttributeValueId,
    pub prop_id: PropId,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValueRequests {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
    pub inserts: Vec<InsertPropertyEditorValueRequest>,
    pub updates: Vec<UpdatePropertyEditorValueRequest>,
}

pub async fn upsert_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<PropertyEditorValueRequests>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut prop_detail: Vec<serde_json::Value> = vec![];

    for update in request.updates {
        // Determine how to update the value based on whether it corresponds to a secret. The vast
        // majority of the time, the request will not be for a secret.
        if request.is_for_secret {
            if let Some(value) = request.value.as_ref() {
                let secret_id: SecretId = serde_json::from_value(value.to_owned())?;
                Secret::attach_for_attribute_value(
                    &ctx,
                    request.attribute_value_id,
                    Some(secret_id),
                )
                .await?;
            } else {
                Secret::attach_for_attribute_value(&ctx, request.attribute_value_id, None).await?;
            }
        } else {
            AttributeValue::update(&ctx, update.attribute_value_id, update.value).await?;
        }

        let prop = Prop::get_by_id_or_error(&ctx, update.prop_id).await?;

        // In this context, there will always be a parent attribute value id
        let parent_prop = if let Some(att_val_id) = update.parent_attribute_value_id {
            if let Some(prop_id) = AttributeValue::prop_id_for_id(&ctx, att_val_id).await? {
                Some(Prop::get_by_id_or_error(&ctx, prop_id).await?)
            } else {
                None
            }
        } else {
            None
        };

        prop_detail.push(serde_json::json!({
            "prop_id": prop.id,
            "prop_name": prop.name,
            "parent_prop_id": parent_prop.as_ref().map(|prop| prop.id),
            "parent_prop_name": parent_prop.as_ref().map(|prop| prop.name.clone()),
        }));
    }

    for insert in request.inserts {
        let _ = AttributeValue::insert(
            &ctx,
            insert.parent_attribute_value_id,
            insert.value,
            insert.key,
        )
        .await?;

        let prop = Prop::get_by_id_or_error(&ctx, insert.prop_id).await?;

        // In this context, there will always be a parent attribute value id
        let parent_prop = if let Some(prop_id) =
            AttributeValue::prop_id_for_id(&ctx, insert.parent_attribute_value_id).await?
        {
            Some(Prop::get_by_id_or_error(&ctx, prop_id).await?)
        } else {
            None
        };

        prop_detail.push(serde_json::json!({
            "prop_id": prop.id,
            "prop_name": prop.name,
            "parent_prop_id": parent_prop.as_ref().map(|prop| prop.id),
            "parent_prop_name": parent_prop.as_ref().map(|prop| prop.name.clone()),
        }));
    }

    // Track
    {
        let component = Component::get_by_id(&ctx, request.component_id).await?;

        let component_schema = component.schema(&ctx).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "property_value_updated",
            serde_json::json!({
                "how": "/component/property_value_updated",
                "component_id": component.id(),
                "component_schema_name": component_schema.name(),
                "change_set_id": ctx.change_set_id(),
                "prop_details": prop_detail,
            }),
        );
    }

    WsEvent::component_updated(&ctx, request.component_id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
