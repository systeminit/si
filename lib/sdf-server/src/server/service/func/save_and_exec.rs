use super::{
    save_func::{do_save_func, SaveFuncRequest, SaveFuncResponse},
    FuncError, FuncResult,
};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    job::definition::DependentValuesUpdate, ActionPrototype, AttributePrototype, AttributeValue,
    AttributeValueError, AttributeValueId, Component, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, PropId, RootPropChild, SchemaVariant, StandardModel,
    ValidationPrototype, WsEvent,
};
use serde_json::Value;
use std::collections::HashMap;

async fn update_values_for_func(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    let prototypes = AttributePrototype::find_for_func(ctx, func.id()).await?;
    if prototypes.is_empty() {
        return Err(FuncError::FuncExecutionFailedNoPrototypes);
    }

    for proto in prototypes {
        let mut values = proto.attribute_values(ctx).await?;

        let mut value_ids = values
            .iter()
            .cloned()
            .map(|av| *av.id())
            .collect::<Vec<AttributeValueId>>();

        for value in values.iter_mut() {
            match value.update_from_prototype_function(ctx).await {
                Ok(_) => {}
                Err(AttributeValueError::FuncBackendResultFailure { message, .. }) => {
                    return Err(FuncError::FuncExecutionFailed(message));
                }
                Err(err) => Err(err)?,
            };

            // We need to make this generic to handle *any* value type so that it creates the
            // child proxies for any value that needs them, but I'm rigging this up just for
            // leaf functions right now.
            if proto.context.is_component_unset()
                && !proto.context.is_prop_unset()
                && matches!(
                    func.backend_response_type(),
                    FuncBackendResponseType::Qualification
                        | FuncBackendResponseType::CodeGeneration
                )
            {
                let parent_attribute_value = match value.parent_attribute_value(ctx).await? {
                    Some(pav) => pav,
                    None => {
                        continue;
                    }
                };

                let schema_variant =
                    match SchemaVariant::find_for_prop(ctx, proto.context.prop_id()).await? {
                        Some(sv) => sv,
                        None => {
                            return Err(FuncError::SchemaVariantNotFoundForProp(
                                proto.context.prop_id(),
                            ))
                        }
                    };

                for component in
                    Component::list_for_schema_variant(ctx, *schema_variant.id()).await?
                {
                    let qualification_attribute_value =
                        Component::root_prop_child_attribute_value_for_component(
                            ctx,
                            *component.id(),
                            match func.backend_response_type() {
                                FuncBackendResponseType::CodeGeneration => RootPropChild::Code,
                                FuncBackendResponseType::Qualification => RootPropChild::Qualification,
                                _ => unreachable!("we guard this with a match above to ensure we only have leaf functions with root prop children")
                            }
                        )
                        .await?;

                    let new_child_proxy_ids = qualification_attribute_value
                        .populate_child_proxies_for_value(
                            ctx,
                            *parent_attribute_value.id(),
                            qualification_attribute_value.context,
                        )
                        .await?;

                    if let Some(new_child_proxy_ids) = new_child_proxy_ids {
                        for new_child_proxy_av_id in new_child_proxy_ids {
                            if let Some(mut proxy_av) =
                                AttributeValue::get_by_id(ctx, &new_child_proxy_av_id).await?
                            {
                                if proxy_av.key() == value.key() {
                                    proxy_av.update_from_prototype_function(ctx).await?;
                                    value_ids.push(new_child_proxy_av_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        ctx.enqueue_job(DependentValuesUpdate::new(
            ctx.access_builder(),
            *ctx.visibility(),
            value_ids,
        ))
        .await?;
    }

    Ok(())
}

async fn run_validations(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    let protos = ValidationPrototype::list_for_func(ctx, *func.id()).await?;
    if protos.is_empty() {
        return Err(FuncError::FuncExecutionFailedNoPrototypes);
    }

    for proto in protos {
        let schema_variant_id = proto.context().schema_variant_id();
        if schema_variant_id.is_none() {
            continue;
        }
        let components = Component::list_for_schema_variant(ctx, schema_variant_id).await?;
        for component in components {
            let mut cache: HashMap<PropId, (Option<Value>, AttributeValue)> = HashMap::new();
            component
                .check_single_validation(ctx, &proto, &mut cache)
                .await?;
        }
    }

    Ok(())
}

async fn run_actions(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    let protos = ActionPrototype::find_for_func(ctx, *func.id()).await?;
    if protos.is_empty() {
        return Err(FuncError::FuncExecutionFailedNoPrototypes);
    }

    for proto in ActionPrototype::find_for_func(ctx, *func.id()).await? {
        let schema_variant_id = proto.context().schema_variant_id();
        if schema_variant_id.is_none() {
            continue;
        }
        let components = Component::list_for_schema_variant(ctx, schema_variant_id).await?;
        for component in components {
            proto.run(ctx, *component.id()).await?;
        }
    }

    Ok(())
}

pub async fn save_and_exec(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<Json<SaveFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let (save_func_response, func) = do_save_func(&ctx, request).await?;

    match func.backend_kind() {
        FuncBackendKind::JsAttribute => {
            update_values_for_func(&ctx, &func).await?;
        }
        FuncBackendKind::JsValidation => {
            run_validations(&ctx, &func).await?;
        }
        FuncBackendKind::JsAction => {
            run_actions(&ctx, &func).await?;
        }
        _ => {}
    }

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(save_func_response))
}
