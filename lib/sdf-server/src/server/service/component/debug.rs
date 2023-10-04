use std::collections::HashMap;
use std::time::Instant;

use axum::extract::Query;
use axum::Json;

use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{
    component::view::{AttributeDebugView, ComponentDebugView},
    AttributeContext, AttributePrototypeArgument, AttributePrototypeId, AttributeValueId,
    Component, ComponentId, DalContext, FuncArgument, FuncId, InternalProvider, Prop, PropId,
    SchemaVariantId, StandardModel, Visibility,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendComponentDebugView {
    name: String,
    schema_variant_id: SchemaVariantId,
    attributes: Vec<FrontendAttributeDebugView>,
    input_sockets: Vec<FrontendAttributeDebugView>,
    output_sockets: Vec<FrontendAttributeDebugView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendAttributeDebugView {
    name: String,
    path: String,
    debug_data: AttributeMetadataView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeMetadataView {
    pub value_id: AttributeValueId,
    pub func_name: String,
    pub func_id: FuncId,
    pub func_args: serde_json::Value,
    pub arg_sources: HashMap<String, Option<String>>,
    pub visibility: Visibility,
    pub value: Option<serde_json::Value>,
    pub prototype_id: AttributePrototypeId,
    pub prototype_context: AttributeContext,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

type DebugComponentResponse = FrontendComponentDebugView;

pub async fn debug_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<DebugComponentRequest>,
) -> ComponentResult<Json<DebugComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentDebug("component not found".into()))?;

    let debug_view = ComponentDebugView::new(&ctx, &component).await?;

    let mut attributes = vec![];
    let mut input_sockets = vec![];
    let mut output_sockets = vec![];

    let transform_start = Instant::now();

    for attribute_debug in debug_view.attributes {
        attributes.push(FrontendAttributeDebugView {
            name: attribute_debug
                .prop
                .as_ref()
                .map(|p| p.name())
                .unwrap_or("")
                .into(),
            path: attribute_debug.path.to_owned(),
            debug_data: get_attribute_metadata(&ctx, attribute_debug).await?,
        });
    }

    for attribute_debug in debug_view.input_sockets {
        input_sockets.push(FrontendAttributeDebugView {
            name: attribute_debug.path.to_owned(),
            path: "Input Socket".into(),
            debug_data: get_attribute_metadata(&ctx, attribute_debug).await?,
        });
    }

    for attribute_debug in debug_view.output_sockets {
        output_sockets.push(FrontendAttributeDebugView {
            name: attribute_debug.path.to_owned(),
            path: "Output Socket".into(),
            debug_data: get_attribute_metadata(&ctx, attribute_debug).await?,
        });
    }

    dbg!(transform_start.elapsed());

    let component_view = DebugComponentResponse {
        name: debug_view.name,
        schema_variant_id: debug_view.schema_variant_id,
        attributes,
        input_sockets,
        output_sockets,
    };

    Ok(Json(component_view))
}

async fn get_attribute_metadata(
    ctx: &DalContext,
    debug_view: AttributeDebugView,
) -> ComponentResult<AttributeMetadataView> {
    let func_args = debug_view.func_binding.args().to_owned();

    let mut arg_sources = HashMap::new();

    for apa in
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *debug_view.prototype.id())
            .await?
    {
        let arg = FuncArgument::get_by_id(ctx, &apa.func_argument_id())
            .await?
            .ok_or(ComponentError::ComponentDebug(format!(
                "could not find func argument {}",
                apa.func_argument_id()
            )))?;

        let internal_provider_id = apa.internal_provider_id();
        let input_ip_name = if internal_provider_id.is_some() {
            let ip = InternalProvider::get_by_id(ctx, &internal_provider_id)
                .await?
                .ok_or(ComponentError::ComponentDebug(format!(
                    "could not find internal provider for input: {}",
                    internal_provider_id
                )))?;

            let prop_id = *ip.prop_id();
            let path = if prop_id == PropId::NONE {
                format!("Input Socket: {}", ip.name())
            } else {
                let prop =
                    Prop::get_by_id(ctx, &prop_id)
                        .await?
                        .ok_or(ComponentError::ComponentDebug(format!(
                            "could not find prop {} for provider for input {}",
                            prop_id, internal_provider_id
                        )))?;

                format!("Prop: /{}", prop.path().with_replaced_sep("/"))
            };

            Some(path)
        } else {
            None
        };

        arg_sources.insert(arg.name().into(), input_ip_name);
    }

    Ok(AttributeMetadataView {
        value_id: *debug_view.attribute_value.id(),
        func_name: debug_view.func.name().into(),
        func_id: debug_view.func.id().to_owned(),
        func_args,
        arg_sources,
        visibility: debug_view.attribute_value.visibility().to_owned(),
        value: debug_view
            .attribute_value
            .get_unprocessed_value(ctx)
            .await?,
        prototype_id: *debug_view.prototype.id(),
        prototype_context: debug_view.prototype.context,
    })
}
