use axum::{extract::Query, Json};
use dal::prop::PROP_PATH_SEPARATOR;
use dal::{
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentId,
    FuncBindingReturnValue, Prop, PropKind, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
            Some(prop_id) => (prop_id, prop.id()),
        };

        let resource_prop_av = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*resource_prop_id),
                internal_provider_id: None,
                external_provider_id: None,
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
                prop_id: Some(*domain_prop_id),
                internal_provider_id: None,
                external_provider_id: None,
                component_id: Some(*component.id()),
            },
        )
        .await?
        .ok_or(ComponentError::AttributeValueNotFound)?;

        let maybe_domain_value =
            FuncBindingReturnValue::get_by_id(ctx, &domain_prop_av.func_binding_return_value_id())
                .await?
                .and_then(|v| v.value().cloned());

        let is_equal = match prop.kind() {
            PropKind::Array | PropKind::Object | PropKind::Map => continue,
            PropKind::Boolean | PropKind::Integer | PropKind::String => {
                maybe_domain_value == maybe_resource_value
            }
        };

        if !is_equal {
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
    }

    Ok(Json(diff_tree))
}
