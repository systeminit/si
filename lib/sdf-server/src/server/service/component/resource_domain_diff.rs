use axum::{extract::Query, Json};
use dal::func::backend::js_reconciliation::{
    ReconciliationDiff, ReconciliationDiffDomain, ReconciliationResult,
};
use dal::prop::PROP_PATH_SEPARATOR;
use dal::{
    AttributeReadContext, AttributeValue, AttributeView, Component, ComponentId,
    ExternalProviderId, Func, FuncBinding, FuncError, InternalProviderId, Prop,
    ReconciliationPrototype, ReconciliationPrototypeContext, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::component::ComponentError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceDomainDiffRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceDomainDiffResponse {
    diff: HashMap<String, ReconciliationDiff>,
    reconciliation: Option<ReconciliationResult>,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct DiffValue {
    diff: bool,
    new_value: Option<serde_json::Value>,
}

pub async fn get_diff(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetResourceDomainDiffRequest>,
) -> ComponentResult<Json<GetResourceDomainDiffResponse>> {
    let ctx = &builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(ctx, &request.component_id)
        .await?
        .ok_or_else(|| ComponentError::ComponentNotFound(request.component_id))?;

    let schema_variant = component
        .schema_variant(ctx)
        .await?
        .ok_or_else(|| ComponentError::SchemaVariantNotFound)?;

    // Check if resource prop has been filled yet
    if component.resource(ctx).await?.payload.is_none() {
        return Ok(Json(GetResourceDomainDiffResponse::default()));
    }

    let props = Prop::find_by_attr(ctx, "schema_variant_id", schema_variant.id()).await?;

    let mut diff = HashMap::new();

    for prop in props {
        let (domain_prop_id, resource_prop_id) = match prop.refers_to_prop_id() {
            None => continue,
            Some(prop_id) => (*prop_id, *prop.id()),
        };

        let context = AttributeReadContext {
            prop_id: Some(resource_prop_id),
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(*component.id()),
        };
        let resource_prop_av = AttributeValue::find_for_context(ctx, context)
            .await?
            .ok_or(ComponentError::AttributeValueNotFound)?;

        let view_context = AttributeReadContext {
            prop_id: None,
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(*component.id()),
        };

        let resource_prop_view =
            AttributeView::new(ctx, view_context, Some(*resource_prop_av.id())).await?;

        let context = AttributeReadContext {
            prop_id: Some(domain_prop_id),
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(*component.id()),
        };

        let domain_prop_av = AttributeValue::find_for_context(ctx, context)
            .await?
            .ok_or(ComponentError::AttributeValueNotFound)?;

        let domain_prop_view =
            AttributeView::new(ctx, view_context, Some(*domain_prop_av.id())).await?;

        if let Some(func_id) = prop.diff_func_id() {
            let func = Func::get_by_id(ctx, func_id)
                .await?
                .ok_or(FuncError::NotFound(*func_id))?;
            let func_binding = FuncBinding::new(
                ctx,
                serde_json::json!({
                    "first": domain_prop_view.value(),
                    "second": resource_prop_view.value(),
                }),
                *func.id(),
                *func.backend_kind(),
            )
            .await?;
            let diff_value = func_binding
                .execute(ctx)
                .await?
                .value()
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            let diff_value = DiffValue::deserialize(&diff_value)?;

            // TODO: Should we treat unset as equal or not?
            if diff_value.diff {
                diff.insert(
                    prop.path().clone().replace(PROP_PATH_SEPARATOR, "/"),
                    ReconciliationDiff {
                        normalized_resource: diff_value.new_value,
                        resource: resource_prop_view.value().clone(),
                        domain: ReconciliationDiffDomain {
                            id: *domain_prop_av.id(),
                            value: domain_prop_view.value().clone(),
                        },
                    },
                );
            }
        } else {
            warn!("Prop {} does not have diff functions set, therefore can't be diffed with prop {domain_prop_id:?}", prop.path());
        }
    }

    let context = ReconciliationPrototypeContext {
        component_id: *component.id(),
        schema_variant_id: *schema_variant.id(),
    };
    let reconciliation = if let Some(reconciliation_prototype) =
        ReconciliationPrototype::find_for_context(ctx, context).await?
    {
        let func = reconciliation_prototype.func(ctx).await?;
        let func_binding = FuncBinding::new(
            ctx,
            serde_json::to_value(&diff)?,
            *func.id(),
            *func.backend_kind(),
        )
        .await?;
        let func_binding_return_value = func_binding.execute(ctx).await?;
        let reconciliation = ReconciliationResult::deserialize(
            func_binding_return_value
                .value()
                .unwrap_or(&serde_json::Value::Null),
        )?;
        Some(reconciliation)
    } else {
        warn!(
            "No reconciliation prototype found for component {} of schema variant {}",
            component.id(),
            schema_variant.id()
        );
        None
    };

    ctx.commit().await?;

    Ok(Json(GetResourceDomainDiffResponse {
        diff,
        reconciliation,
    }))
}
