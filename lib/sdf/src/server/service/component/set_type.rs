use axum::Json;
use dal::attribute::context::AttributeContextBuilder;
use dal::{
    AttributeContext, AttributePrototype, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueId, Component, ExternalProvider, ExternalProviderId, Func,
    FuncBinding, InternalProvider, InternalProviderId, PropId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::component::ComponentError;

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

pub async fn set_type(
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
        request.value.clone(),
        request.key,
    )
    .await?;

    let component_id = request.attribute_context.component_id();

    let component = Component::get_by_id(&ctx, &component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;

    let schema_variant = component
        .schema_variant(&ctx)
        .await?
        .ok_or(ComponentError::SchemaVariantNotFound)?;

    let external_providers =
        ExternalProvider::list_for_schema_variant(&ctx, *schema_variant.id()).await?;

    let internal_providers =
        InternalProvider::list_explicit_for_schema_variant(&ctx, *schema_variant.id()).await?;

    if let Some(value) = request.value {
        if value == "aggregationFrame" {
            let func_id = *Func::find_by_attr(&ctx, "name", &"si:identity")
                .await?
                .first()
                .ok_or(ComponentError::IdentityFuncNotFound)?
                .id();

            let (func_binding, fbrv) = FuncBinding::create_and_execute(
                &ctx,
                serde_json::json![{ "identity": null }],
                func_id,
            )
            .await?;

            for external_provider in external_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(InternalProviderId::NONE),
                    external_provider_id: Some(*external_provider.id()),
                    component_id: Some(component_id),
                };

                let attr_write_context =
                    AttributeContextBuilder::from(attribute_read_context).to_context()?;

                let attribute_value =
                    AttributeValue::find_for_context(&ctx, attribute_read_context)
                        .await?
                        .ok_or(ComponentError::AttributeValueNotFound)?;

                if attribute_value.context.is_component_unset() {
                    AttributePrototype::new(
                        &ctx,
                        func_id,
                        *func_binding.id(),
                        *fbrv.id(),
                        attr_write_context,
                        None,
                        None,
                    )
                    .await?;
                } else {
                    AttributePrototype::new_with_existing_value(
                        &ctx,
                        func_id,
                        attr_write_context,
                        None,
                        None,
                        *attribute_value.id(),
                    )
                    .await?;
                };
            }

            for internal_provider in internal_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(*internal_provider.id()),
                    external_provider_id: Some(ExternalProviderId::NONE),
                    component_id: Some(component_id),
                };

                let attr_write_context =
                    AttributeContextBuilder::from(attribute_read_context).to_context()?;

                let attribute_value =
                    AttributeValue::find_for_context(&ctx, attribute_read_context)
                        .await?
                        .ok_or(ComponentError::AttributeValueNotFound)?;

                let prototype = attribute_value
                    .attribute_prototype(&ctx)
                    .await?
                    .ok_or(ComponentError::AttributePrototypeNotFound)?;

                let arguments = AttributePrototypeArgument::find_by_attr(
                    &ctx,
                    "attribute_prototype_id",
                    prototype.id(),
                )
                .await?;

                let new_prototype = if attribute_value.context.is_component_unset() {
                    AttributePrototype::new(
                        &ctx,
                        func_id,
                        *func_binding.id(),
                        *fbrv.id(),
                        attr_write_context,
                        None,
                        None,
                    )
                    .await?
                } else {
                    AttributePrototype::new_with_existing_value(
                        &ctx,
                        func_id,
                        attr_write_context,
                        None,
                        None,
                        *attribute_value.id(),
                    )
                    .await?
                };

                for argument in arguments {
                    AttributePrototypeArgument::new_for_inter_component(
                        &ctx,
                        *new_prototype.id(),
                        argument.func_argument_id(),
                        argument.head_component_id(),
                        argument.tail_component_id(),
                        argument.external_provider_id(),
                    )
                    .await?;
                }
            }
        } else {
            for external_provider in external_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(InternalProviderId::NONE),
                    external_provider_id: Some(*external_provider.id()),
                    component_id: Some(component_id),
                };

                let attribute_value =
                    AttributeValue::find_for_context(&ctx, attribute_read_context)
                        .await?
                        .ok_or(ComponentError::AttributeValueNotFound)?;

                if !attribute_value.context.is_component_unset() {
                    attribute_value.unset_attribute_prototype(&ctx).await?;
                    attribute_value.delete(&ctx).await?;
                }
            }

            for internal_provider in internal_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(*internal_provider.id()),
                    external_provider_id: Some(ExternalProviderId::NONE),
                    component_id: Some(component_id),
                };

                let attribute_value =
                    AttributeValue::find_for_context(&ctx, attribute_read_context)
                        .await?
                        .ok_or(ComponentError::AttributeValueNotFound)?;

                if !attribute_value.context.is_component_unset() {
                    attribute_value.unset_attribute_prototype(&ctx).await?;
                    attribute_value.delete(&ctx).await?;
                }
            }
        }
    }

    WsEvent::change_set_written(&ctx)
        .await?
        .publish(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(UpdatePropertyEditorValueResponse { success: true }))
}
