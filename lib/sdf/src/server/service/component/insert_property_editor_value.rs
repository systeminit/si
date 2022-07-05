use axum::Json;
use dal::{AttributeContext, AttributeValue, AttributeValueId, Visibility};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertPropertyEditorValueRequest {
    pub parent_attribute_value_id: AttributeValueId,
    pub attribute_context: AttributeContext,
    pub value: Option<serde_json::Value>,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertPropertyEditorValueResponse {
    success: bool,
}

pub async fn insert_property_editor_value(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<InsertPropertyEditorValueRequest>,
) -> ComponentResult<Json<InsertPropertyEditorValueResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    let (_, async_tasks) = AttributeValue::insert_for_context(
        &ctx,
        request.attribute_context,
        request.parent_attribute_value_id,
        request.value,
        request.key,
    )
    .await?;

    txns.commit().await?;

    //tokio::task::spawn(async move {
    //    if let Err(err) = async_tasks
    //        .run(request_ctx, request.visibility, &builder)
    //        .await
    //    {
    //        error!("Component async task execution failed: {err}");
    //    }
    //});

    Ok(Json(InsertPropertyEditorValueResponse { success: true }))
}
