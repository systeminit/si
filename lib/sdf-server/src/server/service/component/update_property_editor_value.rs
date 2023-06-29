use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::{
    AttributeContext, AttributeValue, AttributeValueId, ChangeSet, Component, ComponentId, Prop,
    PropId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::component::ComponentError;

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

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(&ctx).await?, None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    let attribute_context = AttributeContext::builder()
        .set_prop_id(request.prop_id)
        .set_component_id(request.component_id)
        .to_context()?;
    let (_, _) = AttributeValue::update_for_context(
        &ctx,
        request.attribute_value_id,
        request.parent_attribute_value_id,
        attribute_context,
        request.value,
        request.key,
    )
    .await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound(request.component_id))?;

    let component_schema = component
        .schema(&ctx)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?;

    let prop = Prop::get_by_id(&ctx, &request.prop_id)
        .await?
        .ok_or(ComponentError::PropNotFound(request.prop_id))?;

    // In this context, there will always be a parent attribute value id
    let parent_prop = if let Some(att_val_id) = request.parent_attribute_value_id {
        Some(AttributeValue::find_prop_for_value(&ctx, att_val_id).await?)
    } else {
        None
    };

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "property_value_updated",
        serde_json::json!({
            "component_id": component.id(),
            "component_schema_name": component_schema.name(),
            "prop_id": prop.id(),
            "prop_name": prop.name(),
            "parent_prop_id": parent_prop.as_ref().map(|prop| prop.id()),
            "parent_prop_name": parent_prop.as_ref().map(|prop| prop.name()),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
