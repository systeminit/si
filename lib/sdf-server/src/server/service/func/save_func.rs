use std::collections::HashSet;

use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::authentication_prototype::{AuthenticationPrototype, AuthenticationPrototypeContext};
use dal::{
    attribute::context::AttributeContextBuilder,
    func::argument::FuncArgument,
    schema::variant::leaves::{LeafInputLocation, LeafKind},
    ActionKind, ActionPrototype, ActionPrototypeContext, AttributeContext, AttributePrototype,
    AttributePrototypeArgument, AttributePrototypeId, AttributeValue, ChangeSet, Component,
    ComponentId, DalContext, Func, FuncBackendKind, FuncBinding, FuncId, InternalProviderId, Prop,
    SchemaVariantId, StandardModel, Visibility, WsEvent,
};
use dal::{FuncBackendResponseType, PropKind, SchemaVariant};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

use super::{
    AttributePrototypeArgumentView, AttributePrototypeView, FuncArgumentView, FuncAssociations,
    FuncError, FuncResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub display_name: Option<String>,
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
    pub types: String,
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
#[remain::sorted]
enum RemovedPrototypeOp {
    Delete,
    Reset,
}

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototypes: Vec<AttributePrototypeView>,
    removed_protoype_op: RemovedPrototypeOp,
    key: Option<String>,
) -> FuncResult<FuncBackendResponseType> {
    let mut id_set = HashSet::new();
    let mut prop_kind: Option<PropKind> = None;
    let mut computed_backend_response_type = *func.backend_response_type();

    for proto_view in prototypes {
        let context = proto_view.to_attribute_context()?;

        let (mut existing_value_proto, need_to_create) =
            match AttributePrototype::find_for_context_and_key(ctx, context, &key)
                .await?
                .pop()
            {
                Some(existing_proto) => (existing_proto, false),
                None => {
                    let mut context_builder = AttributeContextBuilder::new();

                    if let Some(prop_id) = proto_view.prop_id {
                        context_builder.set_prop_id(prop_id);
                    }

                    if let Some(external_provider_id) = proto_view.external_provider_id {
                        context_builder.set_external_provider_id(external_provider_id);
                    }

                    let default_value_context = context_builder.to_context()?;

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

        if proto.context.prop_id().is_some() {
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

            if matches!(
                computed_backend_response_type,
                FuncBackendResponseType::Json
            ) {
                return Err(FuncError::FuncDestinationPropAndOutputSocket);
            }

            computed_backend_response_type = (*prop.kind()).into();
        } else if proto.context.external_provider_id().is_some() {
            // External and internal providers do not have types yet -- so we set functions that
            // set them to Json, However, some builtins have expressed their type concretely
            // already, so we should continue to use that type to prevent mutation of the function
            // itself. A new function will have an Unset response type, however (until it is bound)
            if prop_kind.is_some() {
                return Err(FuncError::FuncDestinationPropAndOutputSocket);
            }

            if matches!(
                computed_backend_response_type,
                FuncBackendResponseType::Unset,
            ) {
                computed_backend_response_type = FuncBackendResponseType::Json;
            }
        }

        save_attr_func_proto_arguments(ctx, &proto, proto_view.prototype_arguments, need_to_create)
            .await?;
    }

    // TODO: should use a custom query to fetch for *not in* id_set only
    for proto in AttributePrototype::find_for_func(ctx, func.id()).await? {
        if !id_set.contains(proto.id()) {
            match removed_protoype_op {
                RemovedPrototypeOp::Reset => {
                    reset_prototype_and_value_to_intrinsic_function(ctx, &proto, proto.context)
                        .await?
                }
                RemovedPrototypeOp::Delete => {
                    AttributePrototype::remove(ctx, proto.id(), false).await?
                }
            }
        }
    }

    // Unset response type if all bindings removed
    if id_set.is_empty() {
        computed_backend_response_type = FuncBackendResponseType::Unset;
    }

    Ok(computed_backend_response_type)
}

async fn attribute_view_for_leaf_func(
    ctx: &DalContext,
    func: &Func,
    schema_variant_id: SchemaVariantId,
    component_id: Option<ComponentId>,
    inputs: &[LeafInputLocation],
    leaf_kind: LeafKind,
) -> FuncResult<AttributePrototypeView> {
    let existing_proto = SchemaVariant::upsert_leaf_function(
        ctx,
        schema_variant_id,
        component_id,
        leaf_kind,
        inputs,
        func,
    )
    .await?;

    let mut prototype_view = AttributePrototypeView {
        id: AttributePrototypeId::NONE,
        component_id,
        prop_id: if existing_proto.context.prop_id().is_some() {
            Some(existing_proto.context.prop_id())
        } else {
            None
        },
        external_provider_id: if existing_proto.context.external_provider_id().is_some() {
            Some(existing_proto.context.external_provider_id())
        } else {
            None
        },
        prototype_arguments: vec![],
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
            func_argument_name: Some(func_argument.name().to_owned()),
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
    inputs: &[LeafInputLocation],
    leaf_kind: LeafKind,
) -> FuncResult<()> {
    let mut attribute_views = vec![];

    for schema_variant_id in schema_variant_ids {
        attribute_views.push(
            attribute_view_for_leaf_func(ctx, func, schema_variant_id, None, inputs, leaf_kind)
                .await?,
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
                inputs,
                leaf_kind,
            )
            .await?,
        );
    }

    let key = Some(func.name().to_string());

    save_attr_func_prototypes(ctx, func, attribute_views, RemovedPrototypeOp::Delete, key).await?;

    Ok(())
}

async fn reset_prototype_and_value_to_intrinsic_function(
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

async fn save_action_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    kind: Option<ActionKind>,
    schema_variant_ids: Vec<SchemaVariantId>,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();

    let kind = match kind {
        Some(kind) => kind,
        None => {
            if !schema_variant_ids.is_empty() {
                return Err(FuncError::ActionKindMissing(*func.id()));
            }

            ActionKind::Other
        }
    };

    for schema_variant_id in schema_variant_ids {
        let context = ActionPrototypeContext { schema_variant_id };

        let proto = match ActionPrototype::find_for_context_and_func(ctx, context, *func.id())
            .await?
            .pop()
        {
            Some(mut existing_proto) => {
                existing_proto.set_func_id(ctx, *func.id()).await?;
                existing_proto.set_kind_checked(ctx, kind).await?;
                existing_proto
            }
            None => ActionPrototype::new(ctx, *func.id(), kind, context).await?,
        };

        id_set.insert(*proto.id());
    }

    for mut proto in ActionPrototype::find_for_func(ctx, *func.id()).await? {
        if !id_set.contains(proto.id()) {
            proto.delete_by_id(ctx).await?;
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

    func.set_display_name(ctx, request.display_name).await?;
    func.set_name(ctx, request.name).await?;
    func.set_description(ctx, request.description).await?;
    func.set_code_plaintext(ctx, request.code.as_deref())
        .await?;

    match func.backend_kind() {
        FuncBackendKind::JsAction => {
            if let Some(FuncAssociations::Action {
                schema_variant_ids,
                kind,
            }) = request.associations
            {
                save_action_func_prototypes(ctx, &func, kind, schema_variant_ids).await?;
            }
        }
        FuncBackendKind::JsAttribute => match func.backend_response_type() {
            FuncBackendResponseType::CodeGeneration => {
                if let Some(FuncAssociations::CodeGeneration {
                    schema_variant_ids,
                    component_ids,
                    inputs,
                }) = request.associations
                {
                    save_leaf_prototypes(
                        ctx,
                        &func,
                        schema_variant_ids,
                        component_ids,
                        &inputs,
                        LeafKind::CodeGeneration,
                    )
                    .await?;
                }
            }
            FuncBackendResponseType::Qualification => {
                if let Some(FuncAssociations::Qualification {
                    schema_variant_ids,
                    component_ids,
                    inputs,
                }) = request.associations
                {
                    save_leaf_prototypes(
                        ctx,
                        &func,
                        schema_variant_ids,
                        component_ids,
                        &inputs,
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
                    let backend_response_type = save_attr_func_prototypes(
                        ctx,
                        &func,
                        prototypes,
                        RemovedPrototypeOp::Reset,
                        None,
                    )
                    .await?;
                    save_attr_func_arguments(ctx, &func, arguments).await?;

                    func.set_backend_response_type(ctx, backend_response_type)
                        .await?;
                }
            }
        },
        FuncBackendKind::JsAuthentication => {
            if let Some(FuncAssociations::Authentication { schema_variant_ids }) =
                request.associations
            {
                let mut id_set = HashSet::new();

                for schema_variant_id in schema_variant_ids {
                    let proto_context = AuthenticationPrototypeContext { schema_variant_id };

                    let proto = match AuthenticationPrototype::find_for_context_and_func(
                        ctx,
                        &proto_context,
                        *func.id(),
                    )
                    .await?
                    .pop()
                    {
                        None => {
                            AuthenticationPrototype::new(ctx, *func.id(), proto_context).await?
                        }
                        Some(existing_proto) => existing_proto,
                    };

                    id_set.insert(*proto.id());
                }

                for mut proto in AuthenticationPrototype::find_for_func(ctx, *func.id()).await? {
                    if !id_set.contains(proto.id()) {
                        proto.delete_by_id(ctx).await?;
                    }
                }
            }
        }
        FuncBackendKind::Array
        | FuncBackendKind::Boolean
        | FuncBackendKind::Diff
        | FuncBackendKind::Identity
        | FuncBackendKind::Integer
        | FuncBackendKind::JsReconciliation
        | FuncBackendKind::JsSchemaVariantDefinition
        | FuncBackendKind::Map
        | FuncBackendKind::Object
        | FuncBackendKind::String
        | FuncBackendKind::Unset
        | FuncBackendKind::Validation
        | FuncBackendKind::JsValidation => return Err(FuncError::NotWritable),
    }

    let is_revertible = super::is_func_revertible(ctx, &func).await?;
    let view = super::get_func_view(ctx, &func).await?;
    let associations = view.associations;
    let types = view.types;

    Ok((
        SaveFuncResponse {
            associations,
            success: true,
            is_revertible,
            types,
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
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let request_id = request.id;
    let request_associations = request.associations.clone();
    let (save_response, _) = do_save_func(&ctx, request).await?;

    // Track
    {
        let func = Func::get_by_id(&ctx, &request_id)
            .await?
            .ok_or(FuncError::FuncNotFound)?;

        let (component_ids, schema_variant_ids) = match request_associations {
            Some(FuncAssociations::Qualification {
                component_ids,
                schema_variant_ids,
                ..
            })
            | Some(FuncAssociations::CodeGeneration {
                component_ids,
                schema_variant_ids,
                ..
            }) => (component_ids, schema_variant_ids),
            Some(FuncAssociations::Authentication {
                schema_variant_ids, ..
            }) => (vec![], schema_variant_ids),

            None
            | Some(FuncAssociations::Action { .. })
            | Some(FuncAssociations::Attribute { .. })
            | Some(FuncAssociations::SchemaVariantDefinitions { .. })
            | Some(FuncAssociations::Validation { .. }) => (vec![], vec![]),
        };

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "save_func",
            serde_json::json!({
                        "func_id": func.id(),
                        "func_name": func.name(),
                        "func_variant": *func.backend_response_type(),
                        "func_is_builtin": func.builtin(),
                        "func_associated_with_schema_variant_ids": schema_variant_ids,
                        "func_associated_with_component_ids": component_ids,
            }),
        );
        WsEvent::func_saved(&ctx, *func.id())
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(serde_json::to_string(&save_response)?)?)
}
