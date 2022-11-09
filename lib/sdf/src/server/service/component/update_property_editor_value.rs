use axum::Json;
use dal::{AttributeContext, AttributeValue, AttributeValueId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePropertyEditorValueRequest {
    pub attribute_value_id: AttributeValueId,
    pub parent_attribute_value_id: Option<AttributeValueId>,
    pub attribute_context: AttributeContext,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePropertyEditorValueResponse {
    success: bool,
}

pub async fn update_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<UpdatePropertyEditorValueRequest>,
) -> ComponentResult<Json<UpdatePropertyEditorValueResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let (_, _) = AttributeValue::update_for_context(
        &ctx,
        request.attribute_value_id,
        request.parent_attribute_value_id,
        request.attribute_context,
        request.value,
        request.key,
    )
    .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(UpdatePropertyEditorValueResponse { success: true }))
}
