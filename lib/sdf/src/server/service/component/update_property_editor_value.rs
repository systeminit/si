use axum::Json;
use dal::{AttributeContext, AttributeValue, AttributeValueId, Visibility};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<UpdatePropertyEditorValueRequest>,
) -> ComponentResult<Json<UpdatePropertyEditorValueResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    let (_, _, async_tasks) = AttributeValue::update_for_context(
        &ctx,
        request.attribute_value_id,
        request.parent_attribute_value_id,
        request.attribute_context,
        request.value,
        request.key,
    )
    .await?;

    txns.commit().await?;

    if let Some(async_tasks) = async_tasks {
        tokio::task::spawn(async move {
            if let Err(err) = async_tasks
                .run(request_ctx, request.visibility, &builder)
                .await
            {
                error!("Component async task execution failed: {err}");
            }
        });
    }

    Ok(Json(UpdatePropertyEditorValueResponse { success: true }))
}
