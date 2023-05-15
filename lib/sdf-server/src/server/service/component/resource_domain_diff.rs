use axum::{extract::Query, Json};
use dal::prop::PROP_PATH_SEPARATOR;
use dal::{
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentId,
    ExternalProviderId, Func, FuncBinding, FuncBindingReturnValue, FuncError, InternalProvider,
    InternalProviderError, Prop, PropId, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDomainDiffDomain {
    id: AttributeValueId,
    value: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDomainDiff {
    pub resource: Option<Value>,
    pub domain: ResourceDomainDiffDomain,
}

pub type GetResourceDomainDiffResponse = HashMap<String, ResourceDomainDiff>;

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
        return Ok(Json(HashMap::new()));
    }

    let props = Prop::find_by_attr(ctx, "schema_variant_id", schema_variant.id()).await?;

    let mut diff_tree = HashMap::new();

    for prop in props {
        let (domain_prop_id, resource_prop_id) = match prop.refers_to_prop_id() {
            None => continue,
            Some(prop_id) => (*prop_id, *prop.id()),
        };

        let (domain_internal_provider_id, resource_internal_provider_id) = {
            let domain = *InternalProvider::find_for_prop(ctx, domain_prop_id)
                .await?
                .ok_or(InternalProviderError::NotFoundForProp(domain_prop_id))?
                .id();
            let resource = *InternalProvider::find_for_prop(ctx, resource_prop_id)
                .await?
                .ok_or(InternalProviderError::NotFoundForProp(resource_prop_id))?
                .id();
            (domain, resource)
        };

        let resource_prop_av = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(PropId::NONE),
                internal_provider_id: Some(resource_internal_provider_id),
                external_provider_id: Some(ExternalProviderId::NONE),
                component_id: Some(*component.id()),
            },
        )
        .await?
        .ok_or(ComponentError::AttributeValueNotFound)?;

        let maybe_resource_value = FuncBindingReturnValue::get_by_id(
            ctx,
            &resource_prop_av.func_binding_return_value_id(),
        )
        .await?
        .and_then(|v| v.value().cloned());

        let domain_prop_av = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(PropId::NONE),
                internal_provider_id: Some(domain_internal_provider_id),
                external_provider_id: Some(ExternalProviderId::NONE),
                component_id: Some(*component.id()),
            },
        )
        .await?
        .ok_or(ComponentError::AttributeValueNotFound)?;

        let maybe_domain_value =
            FuncBindingReturnValue::get_by_id(ctx, &domain_prop_av.func_binding_return_value_id())
                .await?
                .and_then(|v| v.value().cloned());

        if let Some(func_id) = prop.diff_func_id() {
            let func = Func::get_by_id(ctx, func_id)
                .await?
                .ok_or(FuncError::NotFound(*func_id))?;
            let func_binding = FuncBinding::new(
                ctx,
                serde_json::json!({
                    "first": maybe_domain_value,
                    "second": maybe_resource_value,
                }),
                *func.id(),
                *func.backend_kind(),
            )
            .await?;
            let func_binding_return_value = func_binding.execute(ctx).await?;

            // TODO: Should we treat unset as equal or not?
            if func_binding_return_value.value() != Some(&serde_json::Value::Bool(false)) {
                diff_tree.insert(
                    prop.path().clone().replace(PROP_PATH_SEPARATOR, "/"),
                    ResourceDomainDiff {
                        resource: maybe_resource_value,
                        domain: ResourceDomainDiffDomain {
                            id: *domain_prop_av.id(),
                            value: maybe_domain_value,
                        },
                    },
                );
            }
        } else {
            warn!("Prop {} does not have diff functions set, therefore can't be diffed with prop {domain_prop_id:?}", prop.path());
        }
    }

    Ok(Json(diff_tree))
}
