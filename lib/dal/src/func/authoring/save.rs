//! This module contains [`save_func`] and everything it needs.

use base64::engine::general_purpose;
use base64::Engine;
use std::collections::HashSet;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId};
use crate::func::associations::{FuncArgumentBag, FuncAssociations};
use crate::func::authoring::{FuncAuthoringError, FuncAuthoringResult, RemovedPrototypeOp};
use crate::func::{AttributePrototypeArgumentBag, AttributePrototypeBag, FuncKind};
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::{
    AttributePrototype, AttributePrototypeId, AttributeValue, Component, ComponentId, DalContext,
    DeprecatedActionKind, DeprecatedActionPrototype, Func, FuncBackendResponseType, FuncId,
    SchemaVariant, SchemaVariantId, WorkspaceSnapshotError,
};

pub(crate) async fn save_func(
    ctx: &DalContext,
    id: FuncId,
    display_name: Option<String>,
    name: String,
    description: Option<String>,
    code: Option<String>,
    associations: Option<FuncAssociations>,
) -> FuncAuthoringResult<()> {
    let func = Func::get_by_id_or_error(ctx, id).await?;

    // TODO(nick): we should eventually either return an error or make it a no-op.
    // For now, we need configurable builtins to ensure that the system is working.
    // if func.builtin {
    //     return Err(FuncAuthoringError::NotWritable);
    // }

    Func::modify_by_id(ctx, func.id, |func| {
        func.display_name = display_name.to_owned();
        func.name = name.to_owned();
        func.description = description.to_owned();
        func.code_base64 = code
            .as_ref()
            .map(|code| general_purpose::STANDARD_NO_PAD.encode(code));

        Ok(())
    })
    .await?;

    match func.kind {
        FuncKind::Action => {
            if let Some(FuncAssociations::Action {
                kind,
                schema_variant_ids,
            }) = associations
            {
                save_action_func_prototypes(ctx, &func, kind, schema_variant_ids).await?;
            }
        }
        FuncKind::Attribute => {
            if let Some(FuncAssociations::Attribute {
                prototypes,
                arguments,
            }) = associations
            {
                // First, modify the arguments because they dictate what we can do within the
                // attribute subsystem.
                save_attr_func_arguments(ctx, &func, arguments).await?;

                // Now that we know what func arguments exist and have been modified, we can work
                // within the attribute subsystem.
                let backend_response_type = save_attr_func_prototypes(
                    ctx,
                    &func,
                    prototypes,
                    RemovedPrototypeOp::Reset,
                    None,
                )
                .await?;

                Func::modify_by_id(ctx, func.id, |func| {
                    func.backend_response_type = backend_response_type;
                    Ok(())
                })
                .await?;
            }
        }
        FuncKind::Authentication => {
            if let Some(FuncAssociations::Authentication { schema_variant_ids }) = associations {
                save_auth_func_prototypes(ctx, &func, schema_variant_ids).await?;
            }
        }
        FuncKind::CodeGeneration => {
            if let Some(FuncAssociations::CodeGeneration {
                schema_variant_ids,
                component_ids,
                inputs,
            }) = associations
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
        FuncKind::Qualification => {
            if let Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
                inputs,
            }) = associations
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
        FuncKind::Intrinsic | FuncKind::SchemaVariantDefinition | FuncKind::Unknown => {
            return Err(FuncAuthoringError::NotWritable)
        }
    }

    Ok(())
}

async fn save_action_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    kind: DeprecatedActionKind,
    schema_variant_ids: Vec<SchemaVariantId>,
) -> FuncAuthoringResult<()> {
    for schema_variant_id in schema_variant_ids {
        let prototypes = DeprecatedActionPrototype::for_variant(ctx, schema_variant_id).await?;

        for prototype in prototypes {
            let prototype_func_id = prototype.func_id(ctx).await?;
            if func.id == prototype_func_id {
                DeprecatedActionPrototype::remove(ctx, prototype.id).await?;
                DeprecatedActionPrototype::new(
                    ctx,
                    prototype.name,
                    kind,
                    schema_variant_id,
                    func.id,
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn save_auth_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    schema_variant_ids: Vec<SchemaVariantId>,
) -> FuncAuthoringResult<()> {
    for schema_variant_id in schema_variant_ids {
        let existing_auth_func_ids =
            SchemaVariant::list_auth_func_ids_for_id(ctx, schema_variant_id).await?;

        for existing_auth_func_id in existing_auth_func_ids {
            if func.id == existing_auth_func_id {
                SchemaVariant::remove_authentication_prototype(ctx, func.id, schema_variant_id)
                    .await?;
                SchemaVariant::new_authentication_prototype(ctx, func.id, schema_variant_id)
                    .await?;
            }
        }
    }

    Ok(())
}

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototype_views: Vec<AttributePrototypeBag>,
    removed_protoype_op: RemovedPrototypeOp,
    _key: Option<String>,
) -> FuncAuthoringResult<FuncBackendResponseType> {
    let mut id_set = HashSet::new();
    let mut computed_backend_response_type = func.backend_response_type;

    // Update all prototypes using the func.
    for prototype_view in prototype_views {
        // TODO(nick): don't use the nil id in the future.
        let attribute_prototype_id = if AttributePrototypeId::NONE == prototype_view.id {
            let attribute_prototype = AttributePrototype::new(ctx, func.id).await?;
            attribute_prototype.id
        } else {
            AttributePrototype::update_func_by_id(ctx, prototype_view.id, func.id).await?;
            prototype_view.id
        };
        id_set.insert(attribute_prototype_id);

        save_attr_func_proto_arguments(ctx, prototype_view.id, prototype_view.prototype_arguments)
            .await?;
    }

    // Remove or reset all prototypes not included in the views that use the func.
    for attribute_prototype_id in AttributePrototype::list_ids_for_func_id(ctx, func.id).await? {
        if !id_set.contains(&attribute_prototype_id) {
            remove_or_reset_attr_prototype(ctx, attribute_prototype_id, removed_protoype_op)
                .await?;
        }
    }

    // Use the "unset" response type if all bindings have been removed.
    if id_set.is_empty() {
        computed_backend_response_type = FuncBackendResponseType::Unset;
    }

    Ok(computed_backend_response_type)
}

async fn remove_or_reset_attr_prototype(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
    removed_protoype_op: RemovedPrototypeOp,
) -> FuncAuthoringResult<()> {
    match removed_protoype_op {
        RemovedPrototypeOp::Reset => {
            // NOTE(nick): will there always be an attribute value when resetting? If not,
            // we should not error here.
            let attribute_value_id =
                AttributePrototype::attribute_value_id(ctx, attribute_prototype_id)
                    .await?
                    .ok_or(
                        FuncAuthoringError::AttributeValueNotFoundForAttributePrototype(
                            attribute_prototype_id,
                        ),
                    )?;
            AttributeValue::use_default_prototype(ctx, attribute_value_id).await?
        }
        RemovedPrototypeOp::Delete => {
            AttributePrototype::remove(ctx, attribute_prototype_id).await?
        }
    }
    Ok(())
}

async fn save_attr_func_arguments(
    ctx: &DalContext,
    func: &Func,
    arguments: Vec<FuncArgumentBag>,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();
    for arg in &arguments {
        // TODO(nick): don't use the nil id in the future.
        let func_argument_id = if FuncArgumentId::NONE == arg.id {
            let func_argument =
                FuncArgument::new(ctx, arg.name.as_str(), arg.kind, arg.element_kind, func.id)
                    .await?;
            func_argument.id
        } else {
            FuncArgument::modify_by_id(ctx, arg.id, |existing_arg| {
                existing_arg.name = arg.name.to_owned();
                existing_arg.kind = arg.kind;
                existing_arg.element_kind = arg.element_kind;

                Ok(())
            })
            .await?;
            arg.id
        };

        id_set.insert(func_argument_id);
    }

    for func_arg in FuncArgument::list_for_func(ctx, func.id).await? {
        if !id_set.contains(&func_arg.id) {
            FuncArgument::remove(ctx, func_arg.id).await?;
        }
    }

    Ok(())
}

async fn save_leaf_prototypes(
    ctx: &DalContext,
    func: &Func,
    schema_variant_ids: Vec<SchemaVariantId>,
    component_ids: Vec<ComponentId>,
    inputs: &[LeafInputLocation],
    leaf_kind: LeafKind,
) -> FuncAuthoringResult<()> {
    let mut schema_variant_id_set = HashSet::new();
    schema_variant_id_set.extend(schema_variant_ids);
    for component_id in component_ids {
        schema_variant_id_set.insert(Component::schema_variant_id(ctx, component_id).await?);
    }

    let mut views = Vec::new();
    for schema_variant_id in schema_variant_id_set {
        let attribute_prototype_id = SchemaVariant::upsert_leaf_function(
            ctx,
            schema_variant_id,
            None,
            leaf_kind,
            inputs,
            func,
        )
        .await?;
        views.push(AttributePrototypeBag::assemble(ctx, attribute_prototype_id).await?);
    }

    let key = Some(func.name.to_owned());

    save_attr_func_prototypes(ctx, func, views, RemovedPrototypeOp::Delete, key).await?;

    Ok(())
}

async fn save_attr_func_proto_arguments(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
    arguments: Vec<AttributePrototypeArgumentBag>,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();
    for arg in &arguments {
        // TODO(nick): don't use the nil id in the future.
        if AttributePrototypeArgumentId::NONE != arg.id {
            // The attribute prototype argument may have been deleted when deleting func arguments,
            // so we want to remove or no-op.
            AttributePrototypeArgument::remove_or_no_op(ctx, arg.id).await?;
        }

        // Ensure the func argument exists before continuing.
        if let Err(err) = FuncArgument::get_by_id_or_error(ctx, arg.func_argument_id).await {
            match err {
                FuncArgumentError::WorkspaceSnapshot(
                    WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                        WorkspaceSnapshotGraphError::NodeWithIdNotFound(raw_id),
                    ),
                ) if raw_id == arg.func_argument_id.into() => continue,
                err => return Err(err.into()),
            }
        }

        // NOTE(nick): we always re-create attribute prototype arguments because we do not easily
        // know if the input socket has been changed (or removed) in addition to the func argument
        // existing or moving. However, it is probable that we can refactor this in the future to be
        // more atomic and abstracted while removing foot-guns.
        let new_or_recreated_attribute_prototype_argument =
            AttributePrototypeArgument::new(ctx, attribute_prototype_id, arg.func_argument_id)
                .await?;
        let new_or_recreated_attribute_prototype_argument_id =
            new_or_recreated_attribute_prototype_argument.id();
        if let Some(input_socket_id) = arg.input_socket_id {
            new_or_recreated_attribute_prototype_argument
                .set_value_from_input_socket_id(ctx, input_socket_id)
                .await?;
        }

        id_set.insert(new_or_recreated_attribute_prototype_argument_id);
    }

    for attribute_prototype_argument_id in
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?
    {
        if !id_set.contains(&attribute_prototype_argument_id) {
            AttributePrototypeArgument::remove(ctx, attribute_prototype_argument_id).await?;
        }
    }

    Ok(())
}
