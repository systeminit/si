use axum::extract::OriginalUri;
use axum::Json;
use dal::func::argument::FuncArgumentKind;
use dal::schema::variant::leaves::{LeafInput, LeafInputLocation, LeafKind};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::ValidationPrototypeView;
use super::{
    AttributePrototypeArgumentView, AttributePrototypeView, FuncArgumentView, FuncAssociations,
    FuncError, FuncResult,
};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use dal::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument,
    validation::prototype::context::ValidationPrototypeContext, AttributeContext,
    AttributePrototype, AttributePrototypeArgument, AttributePrototypeId, AttributeValue,
    Component, ComponentId, DalContext, Func, FuncBackendKind, FuncBinding, FuncId,
    InternalProviderId, Prop, SchemaVariantId, StandardModel, Visibility, WsEvent,
};
use dal::{FuncBackendResponseType, PropKind, SchemaVariant, ValidationPrototype};

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
        for mut proto_arg in
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?
        {
            proto_arg.delete_by_id(ctx).await?;
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

    for mut proto_arg in
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?
    {
        if !id_set.contains(proto_arg.id()) {
            proto_arg.delete_by_id(ctx).await?;
        }
    }

    Ok(())
}

/// Determines what we should do with the [`AttributePrototype`](dal::AttributePrototype) and
/// [`AttributeValues`](dal::AttributeValue) that are currently associated with a function but
/// that are having their association removed.
///
/// `RemovedPrototypeOp::Reset` takes the currenty value and resets the prototype to set it to that
/// value using a builtin value function, like `si:setString`, etc.
///
/// `RemovedPrototypeOp::Delete` deletes the prototype and its values.
enum RemovedPrototypeOp {
    Reset,
    Delete,
}

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototypes: Vec<AttributePrototypeView>,
    removed_protoype_op: RemovedPrototypeOp,
    key: Option<String>,
) -> FuncResult<Option<PropKind>> {
    let mut id_set = HashSet::new();
    let mut prop_kind: Option<PropKind> = None;

    for proto_view in prototypes {
        let context = proto_view.to_attribute_context()?;

        let (mut existing_value_proto, need_to_create) =
            match AttributePrototype::find_for_context_and_key(ctx, context, &key)
                .await?
                .pop()
            {
                Some(existing_proto) => (existing_proto, false),
                None => {
                    let default_value_context = AttributeContextBuilder::new()
                        .set_prop_id(proto_view.prop_id)
                        .to_context()?;

                    (
                        AttributePrototype::find_for_context_and_key(
                            ctx,
                            default_value_context,
                            &key,
                        )
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
                key.clone(),
                maybe_parent_attribute_value.map(|mpav| *mpav.id()),
            )
            .await?
        };

        id_set.insert(*proto.id());

        let prop = Prop::get_by_id(ctx, &proto.context.prop_id())
            .await?
            .ok_or(FuncError::PropNotFound)?;
        if let Some(prop_kind) = prop_kind {
            if prop_kind != *prop.kind() {
                return Err(FuncError::FuncDestinationPropKindMismatch);
            }
        } else {
            prop_kind = Some(*prop.kind());
        }

        save_attr_func_proto_arguments(ctx, &proto, proto_view.prototype_arguments, need_to_create)
            .await?;
    }

    // TODO: should use a custom query to fetch for *not in* id_set only
    for proto in AttributePrototype::find_for_func(ctx, func.id()).await? {
        if !id_set.contains(proto.id()) {
            match removed_protoype_op {
                RemovedPrototypeOp::Reset => {
                    reset_prototype_and_value_to_builtin_function(ctx, &proto, proto.context)
                        .await?
                }
                RemovedPrototypeOp::Delete => AttributePrototype::remove(ctx, proto.id()).await?,
            }
        }
    }

    Ok(prop_kind)
}

async fn attribute_view_for_leaf_func(
    ctx: &DalContext,
    func: &Func,
    schema_variant_id: SchemaVariantId,
    component_id: Option<ComponentId>,
    leaf_kind: LeafKind,
) -> FuncResult<AttributePrototypeView> {
    let prop = SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

    let mut prototype_view = AttributePrototypeView {
        id: AttributePrototypeId::NONE,
        component_id,
        prop_id: *prop.id(),
        prototype_arguments: vec![],
    };

    let context = prototype_view.to_attribute_context()?;

    let key = Some(func.name().to_string());

    let existing_proto = match AttributePrototype::find_for_context_and_key(ctx, context, &key)
        .await?
        .pop()
    {
        Some(existing_proto) => existing_proto,
        None => {
            let arg = match FuncArgument::list_for_func(ctx, *func.id()).await?.pop() {
                Some(arg) => arg,
                None => {
                    FuncArgument::new(ctx, "domain", FuncArgumentKind::Object, None, *func.id())
                        .await?
                }
            };

            let (_, new_proto) = SchemaVariant::add_leaf(
                ctx,
                *func.id(),
                schema_variant_id,
                component_id,
                leaf_kind,
                vec![LeafInput {
                    location: LeafInputLocation::Domain,
                    func_argument_id: *arg.id(),
                }],
            )
            .await?;

            new_proto
        }
    };

    let arguments =
        FuncArgument::list_for_func_with_prototype_arguments(ctx, *func.id(), *existing_proto.id())
            .await?;

    let mut argument_views = vec![];

    for (func_argument, maybe_proto_arg) in arguments {
        let proto_arg = maybe_proto_arg.ok_or_else(|| {
            FuncError::FuncArgumentMissingPrototypeArgument(
                *func_argument.id(),
                *existing_proto.id(),
            )
        })?;

        if proto_arg.internal_provider_id() == InternalProviderId::NONE {
            return Err(FuncError::AttributePrototypeMissingInternalProviderId(
                *proto_arg.id(),
            ));
        }

        argument_views.push(AttributePrototypeArgumentView {
            func_argument_id: *func_argument.id(),
            id: Some(*proto_arg.id()),
            internal_provider_id: Some(proto_arg.internal_provider_id()),
        });
    }

    prototype_view.id = *existing_proto.id();
    prototype_view.prototype_arguments = argument_views;

    Ok(prototype_view)
}

async fn save_leaf_prototypes(
    ctx: &DalContext,
    func: &Func,
    schema_variant_ids: Vec<SchemaVariantId>,
    component_ids: Vec<ComponentId>,
    leaf_kind: LeafKind,
) -> FuncResult<()> {
    let mut attribute_views = vec![];

    for schema_variant_id in schema_variant_ids {
        attribute_views.push(
            attribute_view_for_leaf_func(ctx, func, schema_variant_id, None, leaf_kind).await?,
        );
    }

    for component_id in component_ids {
        let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

        attribute_views.push(
            attribute_view_for_leaf_func(
                ctx,
                func,
                schema_variant_id,
                Some(component_id),
                leaf_kind,
            )
            .await?,
        );
    }

    let key = Some(func.name().to_string());

    save_attr_func_prototypes(ctx, func, attribute_views, RemovedPrototypeOp::Delete, key).await?;

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

    for mut proto_arg in
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id()).await?
    {
        proto_arg.delete_by_id(ctx).await?;
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
            if let Some(mut proto) = ValidationPrototype::get_by_id(ctx, proto.id()).await? {
                proto.delete_by_id(ctx).await?;
            }
        }
    }

    Ok(())
}

pub async fn do_save_func(
    ctx: &DalContext,
    request: SaveFuncRequest,
) -> FuncResult<(SaveFuncResponse, Func)> {
    let mut func = Func::get_by_id(ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    // Don't modify builtins, or for other tenancies
    if !ctx.check_tenancy(&func).await? {
        return Err(FuncError::NotWritable);
    }

    func.set_display_name(ctx, Some(request.name)).await?;
    func.set_description(ctx, request.description).await?;
    func.set_handler(ctx, request.handler).await?;
    func.set_code_plaintext(ctx, request.code.as_deref())
        .await?;

    match func.backend_kind() {
        FuncBackendKind::JsValidation => {
            if let Some(FuncAssociations::Validation { prototypes }) = request.associations {
                save_validation_func_prototypes(ctx, &func, prototypes).await?;
            }
        }
        FuncBackendKind::JsAttribute => match func.backend_response_type() {
            FuncBackendResponseType::CodeGeneration => {
                if let Some(FuncAssociations::CodeGeneration {
                    schema_variant_ids,
                    component_ids,
                }) = request.associations
                {
                    save_leaf_prototypes(
                        ctx,
                        &func,
                        schema_variant_ids,
                        component_ids,
                        LeafKind::CodeGeneration,
                    )
                    .await?;
                }
            }
            FuncBackendResponseType::Confirmation => {
                if let Some(FuncAssociations::Confirmation {
                    schema_variant_ids,
                    component_ids,
                }) = request.associations
                {
                    save_leaf_prototypes(
                        ctx,
                        &func,
                        schema_variant_ids,
                        component_ids,
                        LeafKind::Confirmation,
                    )
                    .await?;
                }
            }
            FuncBackendResponseType::Qualification => {
                if let Some(FuncAssociations::Qualification {
                    schema_variant_ids,
                    component_ids,
                }) = request.associations
                {
                    save_leaf_prototypes(
                        ctx,
                        &func,
                        schema_variant_ids,
                        component_ids,
                        LeafKind::Qualification,
                    )
                    .await?;
                }
            }
            _ => {
                if let Some(FuncAssociations::Attribute {
                    prototypes,
                    arguments,
                }) = request.associations
                {
                    let prop_kind = save_attr_func_prototypes(
                        ctx,
                        &func,
                        prototypes,
                        RemovedPrototypeOp::Reset,
                        None,
                    )
                    .await?;
                    save_attr_func_arguments(ctx, &func, arguments).await?;

                    match prop_kind {
                        Some(prop_kind) => {
                            func.set_backend_response_type(
                                ctx,
                                Into::<FuncBackendResponseType>::into(prop_kind),
                            )
                            .await?
                        }
                        // If we're removing all associations there won't be a prop kind
                        None => {
                            func.set_backend_response_type(ctx, FuncBackendResponseType::Unset)
                                .await?
                        }
                    };
                }
            }
        },
        _ => {}
    }

    let is_revertible = super::is_func_revertible(ctx, &func).await?;
    let associations = super::get_func_view(ctx, &func).await?.associations;

    Ok((
        SaveFuncResponse {
            associations,
            success: true,
            is_revertible,
        },
        func,
    ))
}

pub async fn save_func<'a>(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<Json<SaveFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let request_id = request.id;
    let request_associations = request.associations.clone();
    let (save_response, _) = do_save_func(&ctx, request).await?;

    let func = Func::get_by_id(&ctx, &request_id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    // //let (comp_associations, schema_associations) =
    let (component_ids, schema_variant_ids) = match request_associations {
        Some(FuncAssociations::Qualification {
            component_ids,
            schema_variant_ids,
        })
        | Some(FuncAssociations::CodeGeneration {
            component_ids,
            schema_variant_ids,
        })
        | Some(FuncAssociations::Confirmation {
            component_ids,
            schema_variant_ids,
        }) => (component_ids, schema_variant_ids),
        _ => (vec![], vec![]),
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "save_func",
        serde_json::json!({
                    "func_id": func.id(),
                    "func_handler": func.handler().map(|h| h.to_owned()),
                    "func_name": func.name(),
                    "func_variant": *func.backend_response_type(),
                    "func_is_builtin": func.builtin(),
                    "func_associated_with_schema_variant_ids": schema_variant_ids,
                    "func_associated_with_component_ids": component_ids,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(save_response))
}
