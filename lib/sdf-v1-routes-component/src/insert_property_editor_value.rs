use std::collections::HashMap;

use axum::Json;
use dal::{
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    Component,
    ComponentId,
    PropId,
    Visibility,
    WsEvent,
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    HandlerContext,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    ComponentError,
    ComponentResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertPropertyEditorValueRequest {
    pub parent_attribute_value_id: AttributeValueId,
    pub prop_id: PropId,
    pub component_id: ComponentId,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn insert_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<InsertPropertyEditorValueRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    if let Some(key) = &request.key {
        let parent_keys =
            AttributeValue::child_keys_for_id(&ctx, request.parent_attribute_value_id).await?;

        if parent_keys.contains(key) {
            return Err(ComponentError::KeyAlreadyExists(key.to_owned()));
        }
    }

    AttributeValue::insert(
        &ctx,
        request.parent_attribute_value_id,
        request.value,
        request.key,
    )
    .await?;

    let component: Component = Component::get_by_id(&ctx, request.component_id).await?;
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
