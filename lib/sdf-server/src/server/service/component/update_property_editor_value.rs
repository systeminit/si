use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::{
    AttributeValue, AttributeValueId, ChangeSet, Component, ComponentId, Prop, PropId, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePropertyEditorValueRequest {
    pub attribute_value_id: AttributeValueId,
    pub parent_attribute_value_id: Option<AttributeValueId>,
    pub prop_id: PropId,
    pub component_id: ComponentId,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn update_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<UpdatePropertyEditorValueRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    AttributeValue::update(&ctx, request.attribute_value_id, request.value).await?;

    // Track
    {
        let component = Component::get_by_id(&ctx, request.component_id).await?;

        let component_schema = component.schema(&ctx).await?;
        let prop = Prop::get_by_id_or_error(&ctx, request.prop_id).await?;

        // In this context, there will always be a parent attribute value id
        let parent_prop = if let Some(att_val_id) = request.parent_attribute_value_id {
            if let Some(prop_id) = AttributeValue::prop_id_for_id(&ctx, att_val_id).await? {
                Some(Prop::get_by_id_or_error(&ctx, prop_id).await?)
            } else {
                None
            }
        } else {
            None
        };

        track(
            &posthog_client,
            &ctx,
            &original_uri,
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
