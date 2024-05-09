use std::collections::HashSet;
use telemetry::prelude::*;

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
    DeprecatedActionKind, DeprecatedActionPrototype, EdgeWeightKind, Func, FuncBackendResponseType,
    FuncId, OutputSocket, Prop, SchemaVariant, SchemaVariantId, WorkspaceSnapshotError,
};

#[instrument(
    name = "func.authoring.save_func.update_associations",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn update_associations(
    ctx: &DalContext,
    func: &Func,
    associations: FuncAssociations,
) -> FuncAuthoringResult<()> {
    match func.kind {
        FuncKind::Action => match associations {
            FuncAssociations::Action {
                kind,
                schema_variant_ids,
            } => update_action_associations(ctx, func, kind, schema_variant_ids).await,
            invalid => {
                return Err(FuncAuthoringError::InvalidFuncAssociationsForFunc(
                    invalid, func.id, func.kind,
                ))
            }
        },
        FuncKind::Attribute => match associations {
            FuncAssociations::Attribute {
                prototypes,
                arguments,
            } => update_attribute_associations(ctx, func, prototypes, arguments).await,
            invalid => {
                return Err(FuncAuthoringError::InvalidFuncAssociationsForFunc(
                    invalid, func.id, func.kind,
                ))
            }
        },
        FuncKind::Authentication => match associations {
            FuncAssociations::Authentication { schema_variant_ids } => {
                update_authentication_associations(ctx, func, schema_variant_ids).await
            }
            invalid => {
                return Err(FuncAuthoringError::InvalidFuncAssociationsForFunc(
                    invalid, func.id, func.kind,
                ))
            }
        },
        FuncKind::CodeGeneration => match associations {
            FuncAssociations::CodeGeneration {
                schema_variant_ids,
                component_ids,
                inputs,
            } => {
                update_leaf_associations(
                    ctx,
                    func,
                    schema_variant_ids,
                    component_ids,
                    &inputs,
                    LeafKind::CodeGeneration,
                )
                .await
            }
            invalid => Err(FuncAuthoringError::InvalidFuncAssociationsForFunc(
                invalid, func.id, func.kind,
            )),
        },
        FuncKind::Qualification => match associations {
            FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
                inputs,
            } => {
                update_leaf_associations(
                    ctx,
                    func,
                    schema_variant_ids,
                    component_ids,
                    &inputs,
                    LeafKind::Qualification,
                )
                .await
            }
            invalid => Err(FuncAuthoringError::InvalidFuncAssociationsForFunc(
                invalid, func.id, func.kind,
            )),
        },
        kind => Err(FuncAuthoringError::FuncCannotHaveAssociations(
            func.id,
            kind,
            associations,
        )),
    }
}

#[instrument(
    name = "func.authoring.save_func.update_associations.action",
    level = "debug",
    skip(ctx)
)]
async fn update_action_associations(
    ctx: &DalContext,
    func: &Func,
    kind: DeprecatedActionKind,
    schema_variant_ids: Vec<SchemaVariantId>,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();

    // Add the new action prototype to schema variants who do not already have a prototype and
    // replace existing prototypes found.
    for schema_variant_id in schema_variant_ids {
        let existing_action_prototypes =
            DeprecatedActionPrototype::for_variant(ctx, schema_variant_id).await?;

        if existing_action_prototypes.is_empty() {
            DeprecatedActionPrototype::new(
                ctx,
                Some(func.name.to_owned()),
                kind,
                schema_variant_id,
                func.id,
            )
            .await?;
        } else {
            for prototype in existing_action_prototypes {
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

        id_set.insert(schema_variant_id);
    }

    // Remove action prototypes from schema variants that haven't been seen.
    for schema_variant_id in SchemaVariant::list_ids(ctx).await? {
        if !id_set.contains(&schema_variant_id) {
            let action_prototypes_to_be_deleted =
                DeprecatedActionPrototype::for_variant(ctx, schema_variant_id).await?;
            for prototype in action_prototypes_to_be_deleted {
                let prototype_func_id = prototype.func_id(ctx).await?;
                if func.id == prototype_func_id {
                    DeprecatedActionPrototype::remove(ctx, prototype.id).await?;
                }
            }
        }
    }

    Ok(())
}

#[instrument(
    name = "func.authoring.save_func.update_associations.attribute",
    level = "debug",
    skip(ctx)
)]
async fn update_attribute_associations(
    ctx: &DalContext,
    func: &Func,
    prototypes: Vec<AttributePrototypeBag>,
    arguments: Vec<FuncArgumentBag>,
) -> FuncAuthoringResult<()> {
    // First, modify the arguments because they dictate what we can do within the
    // attribute subsystem.
    save_attr_func_arguments(ctx, func, arguments).await?;

    // Now that we know what func arguments exist and have been modified, we can work
    // within the attribute subsystem.
    let backend_response_type =
        save_attr_func_prototypes(ctx, func, prototypes, RemovedPrototypeOp::Reset, None).await?;

    Func::modify_by_id(ctx, func.id, |func| {
        func.backend_response_type = backend_response_type;
        Ok(())
    })
    .await?;

    Ok(())
}

#[instrument(
    name = "func.authoring.save_func.update_associations.authentication",
    level = "debug",
    skip(ctx)
)]
async fn update_authentication_associations(
    ctx: &DalContext,
    func: &Func,
    schema_variant_ids: Vec<SchemaVariantId>,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();

    // Add the new authentication prototype to schema variants who do not already have a prototype.
    // We do not need to re-create or edit the prototypes that already exist because the prototype
    // is merely an edge.
    for schema_variant_id in schema_variant_ids {
        let existing_auth_func_ids =
            SchemaVariant::list_auth_func_ids_for_id(ctx, schema_variant_id).await?;
        if existing_auth_func_ids.is_empty() {
            SchemaVariant::new_authentication_prototype(ctx, func.id, schema_variant_id).await?;
        }
        id_set.insert(schema_variant_id);
    }

    // Remove authentication prototypes from schema variants that haven't been seen.
    for schema_variant_id in
        SchemaVariant::list_schema_variant_ids_using_auth_func_id(ctx, func.id).await?
    {
        if !id_set.contains(&schema_variant_id) {
            SchemaVariant::remove_authentication_prototype(ctx, func.id, schema_variant_id).await?;
        }
    }

    Ok(())
}

#[instrument(
    name = "func.authoring.save_func.update_associations.leaf",
    level = "debug",
    skip(ctx)
)]
async fn update_leaf_associations(
    ctx: &DalContext,
    func: &Func,
    schema_variant_ids: Vec<SchemaVariantId>,
    component_ids: Vec<ComponentId>,
    inputs: &[LeafInputLocation],
    leaf_kind: LeafKind,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();

    // Populate the id set with the provided schema variant ids as well as the schema variant ids
    // for the provided components.
    id_set.extend(schema_variant_ids);
    for component_id in component_ids {
        id_set.insert(Component::schema_variant_id(ctx, component_id).await?);
    }

    let mut views = Vec::new();
    for schema_variant_id in id_set {
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

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototype_bags: Vec<AttributePrototypeBag>,
    removed_protoype_op: RemovedPrototypeOp,
    _key: Option<String>,
) -> FuncAuthoringResult<FuncBackendResponseType> {
    let mut id_set = HashSet::new();
    let mut computed_backend_response_type = func.backend_response_type;

    // Update all prototypes using the func.
    for prototype_bag in prototype_bags {
        // TODO(nick): don't use the nil id in the future.
        let attribute_prototype_id = if AttributePrototypeId::NONE == prototype_bag.id {
            create_new_attribute_prototype(ctx, &prototype_bag, func.id).await?
        } else {
            AttributePrototype::update_func_by_id(ctx, prototype_bag.id, func.id).await?;
            prototype_bag.id
        };
        id_set.insert(attribute_prototype_id);

        // Use the attribute prototype id variable rather than the one off the iterator so that we
        // don't use the nil one by accident.
        save_attr_func_proto_arguments(
            ctx,
            attribute_prototype_id,
            prototype_bag.prototype_arguments,
        )
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
                arg.name.clone_into(&mut existing_arg.name);
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

async fn save_attr_func_proto_arguments(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
    arguments: Vec<AttributePrototypeArgumentBag>,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();

    for arg in &arguments {
        // Ensure that the user is not also requesting a new func argument inside the attribute
        // prototype argument request. They should use the func argument bag to do that.
        if arg.func_argument_id == FuncArgumentId::NONE {
            return Err(FuncAuthoringError::FuncArgumentMustExist(arg.id));
        }

        // Ensure the func argument exists before continuing. By continuing, we will not add the
        // attribute prototype to the id set and will be deleted.
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

        // Always remove and recreate the argument because the func argument or input socket
        // could have changed.
        if AttributePrototypeArgumentId::NONE != arg.id {
            AttributePrototypeArgument::remove_or_no_op(ctx, arg.id).await?;
        }

        let attribute_prototype_argument =
            AttributePrototypeArgument::new(ctx, attribute_prototype_id, arg.func_argument_id)
                .await?;
        let attribute_prototype_argument_id = attribute_prototype_argument.id();

        if let Some(input_socket_id) = arg.input_socket_id {
            attribute_prototype_argument
                .set_value_from_input_socket_id(ctx, input_socket_id)
                .await?;
        } else if let Some(prop_id) = arg.prop_id {
            attribute_prototype_argument
                .set_value_from_prop_id(ctx, prop_id)
                .await?;
        } else {
            return Err(FuncAuthoringError::NoInputLocationGiven(
                attribute_prototype_id,
                arg.func_argument_id,
            ));
        }

        id_set.insert(attribute_prototype_argument_id);
    }

    for attribute_prototype_argument_id in
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?
    {
        if !id_set.contains(&attribute_prototype_argument_id) {
            AttributePrototypeArgument::remove_or_no_op(ctx, attribute_prototype_argument_id)
                .await?;
        }
    }

    Ok(())
}

async fn create_new_attribute_prototype(
    ctx: &DalContext,
    prototype_bag: &AttributePrototypeBag,
    func_id: FuncId,
) -> FuncAuthoringResult<AttributePrototypeId> {
    let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

    let mut affected_attribute_value_ids = Vec::new();

    if let Some(prop_id) = prototype_bag.prop_id {
        if let Some(component_id) = prototype_bag.component_id {
            let attribute_value_ids = Prop::attribute_values_for_prop_id(ctx, prop_id).await?;

            for attribute_value_id in attribute_value_ids {
                if component_id == AttributeValue::component_id(ctx, attribute_value_id).await? {
                    AttributeValue::set_component_prototype_id(
                        ctx,
                        attribute_value_id,
                        attribute_prototype.id,
                    )
                    .await?;
                    affected_attribute_value_ids.push(attribute_value_id);
                }
            }
        } else {
            Prop::add_edge_to_attribute_prototype(
                ctx,
                prop_id,
                attribute_prototype.id,
                EdgeWeightKind::Prototype(None),
            )
            .await?;
        }
    } else if let Some(output_socket_id) = prototype_bag.output_socket_id {
        if let Some(component_id) = prototype_bag.component_id {
            let attribute_value_ids =
                OutputSocket::attribute_values_for_output_socket_id(ctx, output_socket_id).await?;
            for attribute_value_id in attribute_value_ids {
                if component_id == AttributeValue::component_id(ctx, attribute_value_id).await? {
                    AttributeValue::set_component_prototype_id(
                        ctx,
                        attribute_value_id,
                        attribute_prototype.id,
                    )
                    .await?;
                    affected_attribute_value_ids.push(attribute_value_id);
                }
            }
        } else {
            OutputSocket::add_edge_to_attribute_prototype(
                ctx,
                output_socket_id,
                attribute_prototype.id,
                EdgeWeightKind::Prototype(None),
            )
            .await?;
        }
    } else {
        return Err(FuncAuthoringError::NoOutputLocationGiven(func_id));
    }

    if !affected_attribute_value_ids.is_empty() {
        ctx.enqueue_dependent_values_update(affected_attribute_value_ids)
            .await?;
    }

    Ok(attribute_prototype.id)
}
