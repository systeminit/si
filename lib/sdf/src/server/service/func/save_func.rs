use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::ValidationPrototypeView;
use super::{
    AttributePrototypeArgumentView, AttributePrototypeView, FuncArgumentView, FuncAssociations,
    FuncError, FuncResult,
};
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::attribute::context::AttributeContextBuilder;
use dal::validation::prototype::context::ValidationPrototypeContext;
use dal::{
    func::argument::FuncArgument,
    prototype_context::{
        associate_prototypes, HasPrototypeContext, PrototypeContextError, PrototypeContextField,
    },
    AttributeContext, AttributePrototype, AttributePrototypeArgument, AttributeValue,
    ConfirmationPrototype, DalContext, Func, FuncBackendKind, FuncBinding, FuncId,
    PrototypeListForFunc, StandardModel, Visibility, WsEvent,
};
use dal::{SchemaVariant, ValidationPrototype};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub handler: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub associations: Option<FuncAssociations>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncResponse {
    pub associations: Option<FuncAssociations>,
    pub success: bool,
    pub is_revertible: bool,
}

async fn save_attr_func_proto_arguments(
    ctx: &DalContext,
    proto: &AttributePrototype,
    arguments: Vec<AttributePrototypeArgumentView>,
    create_all: bool,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();

    if create_all {
        for proto_arg in
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?
        {
            proto_arg.delete(ctx).await?;
        }
    }

    for arg in &arguments {
        if let Some(arg_id) = arg.id {
            let proto_arg = if arg_id.is_none() || create_all {
                match arg.internal_provider_id {
                    Some(internal_provider_id) => Some(
                        AttributePrototypeArgument::new_for_intra_component(
                            ctx,
                            *proto.id(),
                            arg.func_argument_id,
                            internal_provider_id,
                        )
                        .await?,
                    ),
                    None => None, // This should probably be an error
                }
            } else {
                Some(
                    AttributePrototypeArgument::get_by_id(ctx, &arg_id)
                        .await?
                        .ok_or_else(|| {
                            FuncError::AttributePrototypeMissingArgument(*proto.id(), arg_id)
                        })?,
                )
            };

            if let Some(mut proto_arg) = proto_arg {
                if proto_arg.attribute_prototype_id() != *proto.id() {
                    proto_arg
                        .set_attribute_prototype_id(ctx, *proto.id())
                        .await?;
                }

                if let Some(internal_provider_id) = arg.internal_provider_id {
                    if internal_provider_id != proto_arg.internal_provider_id() {
                        proto_arg
                            .set_internal_provider_id_safe(ctx, internal_provider_id)
                            .await?;
                    }
                }

                let proto_arg_id = *proto_arg.id();
                id_set.insert(proto_arg_id);
            }
        } else if let Some(internal_provider_id) = arg.internal_provider_id {
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *proto.id(),
                arg.func_argument_id,
                internal_provider_id,
            )
            .await?;
        } // else condition should be error here? (saving an arg that has no internal provider id)
    }

    for proto_arg in
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?
    {
        if !id_set.contains(proto_arg.id()) {
            proto_arg.delete(ctx).await?;
        }
    }

    Ok(())
}

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototypes: Vec<AttributePrototypeView>,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();
    for proto_view in prototypes {
        let context = proto_view.to_attribute_context()?;

        let (mut existing_value_proto, need_to_create) =
            match AttributePrototype::find_for_context(ctx, context)
                .await?
                .pop()
            {
                Some(existing_proto) => (existing_proto, false),
                None => {
                    let default_value_context = AttributeContextBuilder::new()
                        .set_prop_id(proto_view.prop_id)
                        .to_context()?;

                    (
                        AttributePrototype::find_for_context(ctx, default_value_context)
                            .await?
                            .pop()
                            .ok_or(FuncError::AttributePrototypeMissing)?,
                        true,
                    )
                }
            };

        let proto = if !need_to_create {
            existing_value_proto.set_func_id(ctx, *func.id()).await?;
            existing_value_proto
        } else {
            let existing_value = existing_value_proto
                .attribute_values(ctx)
                .await?
                .pop()
                .ok_or(FuncError::AttributeValueMissing)?;

            let maybe_parent_attribute_value = existing_value.parent_attribute_value(ctx).await?;

            let (mut func_binding, fbrv) = FuncBinding::create_with_existing_value(
                ctx,
                serde_json::json!({}),
                existing_value.get_value(ctx).await?,
                *func.id(),
            )
            .await?;

            // Clear out the function sha so we know to execute this on the first run in
            // AttributeValue::update_from_prototype_function
            func_binding.set_code_sha256(ctx, "0").await?;

            AttributePrototype::new(
                ctx,
                *func.id(),
                *func_binding.id(),
                *fbrv.id(),
                context,
                None,
                maybe_parent_attribute_value.map(|mpav| *mpav.id()),
            )
            .await?
        };

        id_set.insert(*proto.id());

        save_attr_func_proto_arguments(ctx, &proto, proto_view.prototype_arguments, need_to_create)
            .await?;
    }

    // TODO: should use a custom query to fetch for *not in* id_set only
    for proto in AttributePrototype::find_for_func(ctx, func.id()).await? {
        if !id_set.contains(proto.id()) {
            reset_prototype_and_value_to_builtin_function(ctx, &proto, proto.context).await?
        }
    }

    Ok(())
}

async fn reset_prototype_and_value_to_builtin_function(
    ctx: &DalContext,
    proto: &AttributePrototype,
    context: AttributeContext,
) -> FuncResult<()> {
    let existing_value = proto
        .attribute_values(ctx)
        .await?
        .pop()
        .ok_or(FuncError::AttributeValueMissing)?;

    let maybe_parent_attribute_value = existing_value.parent_attribute_value(ctx).await?;
    let value_value = existing_value.get_value(ctx).await?;

    for proto_arg in
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?
    {
        proto_arg.delete(ctx).await?;
    }

    // This should reset the prototype to a builtin value function
    AttributeValue::update_for_context(
        ctx,
        *existing_value.id(),
        maybe_parent_attribute_value.map(|pav| *pav.id()),
        context,
        value_value,
        proto.key().map(|key| key.to_string()),
    )
    .await?;

    Ok(())
}

async fn save_attr_func_arguments(
    ctx: &DalContext,
    func: &Func,
    arguments: Vec<FuncArgumentView>,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();
    for arg in &arguments {
        let arg_id = if arg.id.is_some() {
            id_set.insert(arg.id);
            let mut existing = FuncArgument::get_by_id(ctx, &arg.id)
                .await?
                .ok_or(FuncError::FuncArgNotFound)?;
            existing.set_name(ctx, &arg.name).await?;
            existing.set_kind(ctx, arg.kind).await?;
            existing.set_element_kind(ctx, arg.element_kind).await?;

            *existing.id()
        } else {
            let new_arg =
                FuncArgument::new(ctx, &arg.name, arg.kind, arg.element_kind, *func.id()).await?;
            *new_arg.id()
        };

        id_set.insert(arg_id);
    }

    for func_arg in FuncArgument::list_for_func(ctx, *func.id()).await? {
        if !id_set.contains(func_arg.id()) {
            FuncArgument::remove(ctx, func_arg.id()).await?;
        }
    }

    Ok(())
}

async fn save_validation_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototypes: Vec<ValidationPrototypeView>,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();

    for proto_view in prototypes {
        let mut context = ValidationPrototypeContext::builder();
        let schema_id = *SchemaVariant::get_by_id(ctx, &proto_view.schema_variant_id)
            .await?
            .ok_or(FuncError::ValidationPrototypeMissingSchemaVariant(
                proto_view.schema_variant_id,
            ))?
            .schema(ctx)
            .await?
            .ok_or(FuncError::ValidationPrototypeMissingSchema)?
            .id();

        let context = context
            .set_prop_id(proto_view.prop_id)
            .set_schema_variant_id(proto_view.schema_variant_id)
            .set_schema_id(schema_id)
            .to_context(ctx)
            .await?;

        let proto = match ValidationPrototype::find_for_context(ctx, context.clone())
            .await?
            .pop()
        {
            Some(mut existing_proto) => {
                existing_proto.set_func_id(ctx, *func.id()).await?;
                existing_proto
            }
            None => {
                ValidationPrototype::new(ctx, *func.id(), serde_json::json!(null), context).await?
            }
        };

        id_set.insert(*proto.id());
    }

    for proto in ValidationPrototype::list_for_func(ctx, *func.id()).await? {
        if !id_set.contains(proto.id()) {
            if let Some(proto) = ValidationPrototype::get_by_id(ctx, proto.id()).await? {
                proto.delete(ctx).await?;
            }
        }
    }

    Ok(())
}

pub async fn save_func<'a>(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<Json<SaveFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    // Don't modify builtins, or for other tenancies
    if !ctx.check_tenancy(&func).await? {
        return Err(FuncError::NotWritable);
    }

    func.set_display_name(&ctx, Some(request.name)).await?;
    func.set_description(&ctx, request.description).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;

    let func_id_copy = *func.id();

    match func.backend_kind() {
        FuncBackendKind::JsValidation => {
            if let Some(FuncAssociations::Validation { prototypes }) = request.associations {
                save_validation_func_prototypes(&ctx, &func, prototypes).await?;
            }
        }
        FuncBackendKind::JsConfirmation => {
            let mut associations: Vec<PrototypeContextField> = vec![];
            if let Some(FuncAssociations::Confirmation {
                schema_variant_ids,
                component_ids,
            }) = request.associations
            {
                associations.append(&mut schema_variant_ids.iter().map(|f| (*f).into()).collect());
                associations.append(&mut component_ids.iter().map(|f| (*f).into()).collect());

                let create_prototype_closure =
                    move |ctx: DalContext, context_field: PrototypeContextField| async move {
                        let func = Func::get_by_id(&ctx, &func_id_copy)
                            .await?
                            .ok_or(PrototypeContextError::FuncNotFound(func_id_copy))?;
                        ConfirmationPrototype::new(
                            &ctx,
                            func.display_name().unwrap_or("unknown"),
                            func_id_copy,
                            ConfirmationPrototype::new_context_for_context_field(context_field),
                        )
                        .await?;

                        Ok(())
                    };

                associate_prototypes(
                    &ctx,
                    &ConfirmationPrototype::list_for_func(&ctx, *func.id()).await?,
                    &associations,
                    Box::new(create_prototype_closure),
                )
                .await?;
            }
        }
        FuncBackendKind::JsAttribute => {
            if let Some(FuncAssociations::Attribute {
                prototypes,
                arguments,
            }) = request.associations
            {
                save_attr_func_prototypes(&ctx, &func, prototypes).await?;
                save_attr_func_arguments(&ctx, &func, arguments).await?;
            }
        }
        _ => {}
    }

    let is_revertible = super::is_func_revertible(&ctx, &func).await?;
    let associations = super::get_func_view(&ctx, &func).await?.associations;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;
    ctx.commit().await?;

    Ok(Json(SaveFuncResponse {
        associations,
        success: true,
        is_revertible,
    }))
}
