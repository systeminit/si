use axum::Json;
use dal::{
    AttributeContext, AttributeReadContext, AttributeValue, AttributeValueId, ExternalProviderId,
    InternalProviderId, StandardModel, SystemId, Visibility, WsEvent,
};
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

    let read_context = AttributeReadContext {
        prop_id: Some(request.attribute_context.prop_id()),
        internal_provider_id: Some(InternalProviderId::NONE),
        external_provider_id: Some(ExternalProviderId::NONE),
        schema_id: Some(request.attribute_context.schema_id()),
        schema_variant_id: Some(request.attribute_context.schema_variant_id()),
        component_id: Some(request.attribute_context.component_id()),
        system_id: Some(SystemId::NONE),
    };

    // When we set the value for the first time, we might only have the schema variant specific
    // value id since there may not yet be a value for the component context. Let's make sure a
    // value does not already exist for this context first. And if it does, set *that* value
    // instead.
    let (av_id, maybe_pav_id) =
        if let Some(av) = AttributeValue::find_for_context(&ctx, read_context).await? {
            if *av.id() != request.attribute_value_id {
                (
                    *av.id(),
                    av.parent_attribute_value(&ctx).await?.map(|pav| *pav.id()),
                )
            } else {
                (
                    request.attribute_value_id,
                    request.parent_attribute_value_id,
                )
            }
        } else {
            (
                request.attribute_value_id,
                request.parent_attribute_value_id,
            )
        };

    let (_, _) = AttributeValue::update_for_context(
        &ctx,
        av_id,
        maybe_pav_id,
        request.attribute_context,
        request.value,
        request.key,
    )
    .await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(UpdatePropertyEditorValueResponse { success: true }))
}
