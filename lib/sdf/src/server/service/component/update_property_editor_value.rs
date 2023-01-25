use axum::Json;
use dal::{
    AttributeContext, AttributeValue, AttributeValueId, ComponentId, PropId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePropertyEditorValueResponse {
    pub success: bool,
}

pub async fn update_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<UpdatePropertyEditorValueRequest>,
) -> ComponentResult<Json<UpdatePropertyEditorValueResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

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

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(UpdatePropertyEditorValueResponse { success: true }))
}
