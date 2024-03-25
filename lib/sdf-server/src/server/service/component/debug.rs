use std::time::Instant;

use axum::extract::Query;
use axum::Json;

use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::attribute::value::debug::AttributeDebugView;
use dal::component::debug::ComponentDebugView;
use dal::{AttributeValueId, Component, ComponentId, SchemaVariantId, Visibility};
use telemetry::prelude::*;

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
    debug_data: AttributeDebugView,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

type DebugComponentResponse = FrontendComponentDebugView;
#[instrument(level = "debug", skip_all)]
pub async fn debug_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<DebugComponentRequest>,
) -> ComponentResult<Json<DebugComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let component = Component::get_by_id(&ctx, request.component_id).await?;
    let schema_variant_id = Component::schema_variant_id(&ctx, component.id()).await?;

    let debug_view = ComponentDebugView::new(&ctx, &component).await?;

    let mut attributes = vec![];
    let input_sockets = vec![];
    let output_sockets = vec![];

    let transform_start = Instant::now();
    let mut cache: Vec<AttributeValueId> = vec![];
    //grab attribute value debug views
    for (av, children) in debug_view.attribute_tree {
        if !cache.contains(&av) {
            let avd = AttributeDebugView::new(&ctx, av, None, None).await?;
            if let Some(name) = avd.prop.as_ref().map(|p| p.name.to_owned()) {
                attributes.push(FrontendAttributeDebugView {
                    name,
                    path: avd.path.to_owned(),
                    debug_data: avd,
                });
                cache.push(av);
            }
        }

        for child_av in children {
            if !cache.contains(&child_av) {
                let child_avd = AttributeDebugView::new(&ctx, child_av, None, Some(av)).await?;
                if let Some(name) = child_avd.prop.as_ref().map(|p| p.name.to_owned()) {
                    attributes.push(FrontendAttributeDebugView {
                        name,
                        path: child_avd.path.to_owned(),
                        debug_data: child_avd,
                    });
                    cache.push(child_av);
                }
            }
        }
    }
    attributes.sort_by_key(|view| view.path.to_lowercase());

    //grab input socket info next
    //grab output socket info next

    dbg!(transform_start.elapsed());

    let component_view = DebugComponentResponse {
        name: component.name(&ctx).await?,
        schema_variant_id,
        attributes,
        input_sockets,
        output_sockets,
    };

    Ok(Json(component_view))
}
