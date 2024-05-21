use std::collections::HashSet;
use telemetry::prelude::*;

use crate::action::prototype::{ActionKind, ActionPrototype};
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId};
use crate::func::associations::{FuncArgumentBag, FuncAssociations};
use crate::func::authoring::{FuncAuthoringError, FuncAuthoringResult};
use crate::func::{AttributePrototypeArgumentBag, AttributePrototypeBag, FuncKind};
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::{
    ActionPrototypeId, AttributePrototype, AttributePrototypeId, AttributeValue, Component,
    ComponentId, DalContext, DeprecatedActionKind, DeprecatedActionPrototype, EdgeWeightKind, Func,
    FuncBackendResponseType, FuncId, OutputSocket, Prop, SchemaVariant, SchemaVariantId,
    WorkspaceSnapshotError,
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
    let mut deprecated_prototypes_to_remove: Vec<ActionPrototypeId> = vec![];
    let mut prototyes_to_remove: Vec<ActionPrototypeId> = vec![];

    for schema_variant_id in SchemaVariant::list_ids(ctx).await? {
        let current_deprecated_prototypes =
            DeprecatedActionPrototype::for_variant(ctx, schema_variant_id).await?;
        for prototype in current_deprecated_prototypes.clone() {
            let prototype_func_id = prototype.func_id(ctx).await?;
            if func.id == prototype_func_id {
                if prototype.kind != kind && kind != DeprecatedActionKind::Other {
                    let existing_kind = current_deprecated_prototypes
                        .clone()
                        .into_iter()
                        .find(|ap| ap.kind == kind);
                    if existing_kind.is_some() {
                        return Err(FuncAuthoringError::KindAlreadyExists(ActionKind::from(
                            kind,
                        )));
                    }
                }
                deprecated_prototypes_to_remove.push(prototype.id);
            }
        }
        let new_action_protypes_for_schema_variant =
            ActionPrototype::for_variant(ctx, schema_variant_id).await?;
        for prototype in new_action_protypes_for_schema_variant.clone() {
            let prototype_func_id = ActionPrototype::func_id(ctx, prototype.id()).await?;
            if func.id == prototype_func_id {
                if prototype.kind != ActionKind::from(kind)
                    && ActionKind::from(kind) != ActionKind::Manual
                {
                    let existing_kind = new_action_protypes_for_schema_variant
                        .clone()
                        .into_iter()
                        .find(|ap| ap.kind == ActionKind::from(kind));
                    if existing_kind.is_some() {
                        return Err(FuncAuthoringError::KindAlreadyExists(ActionKind::from(
                            kind,
                        )));
                    }
                }
                prototyes_to_remove.push(prototype.id());
            }
        }
    }

    for removal_id in deprecated_prototypes_to_remove {
        DeprecatedActionPrototype::remove(ctx, removal_id).await?;
    }

    for removal_id in prototyes_to_remove {
        ActionPrototype::remove(ctx, removal_id).await?;
    }

    // Create or re-create the prototype for the schema variant ids passed in.
    for schema_variant_id in schema_variant_ids {
        DeprecatedActionPrototype::new(
            ctx,
            Some(func.name.to_owned()),
            kind,
            schema_variant_id,
            func.id,
        )
        .await?;

        ActionPrototype::new(
            ctx,
            crate::action::prototype::ActionKind::from(kind),
            func.name.to_owned(),
            None,
            schema_variant_id,
            func.id,
        )
        .await?;
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
        save_attr_func_prototypes(ctx, func, prototypes, true, None).await?;

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

        if !existing_auth_func_ids.iter().any(|id| *id == func.id) {
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
        // TODO(nick): destroy nilId. Log a warning at the moment in case the frontend sends value
        // for no-ops. I will come back and destroy nil id soon.
        if component_id == ComponentId::NONE {
            warn!("skipping component id set to nil id");
        } else {
            id_set.insert(Component::schema_variant_id(ctx, component_id).await?);
        }
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

    save_attr_func_prototypes(ctx, func, views, false, key).await?;

    Ok(())
}

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototype_bags: Vec<AttributePrototypeBag>,
    attempt_to_reset_prototype: bool,
    key: Option<String>,
) -> FuncAuthoringResult<FuncBackendResponseType> {
    let mut id_set = HashSet::new();
    let mut computed_backend_response_type = func.backend_response_type;

    // Update all prototypes using the func.
    for prototype_bag in prototype_bags {
        // TODO(nick): don't use the nil id in the future.
        let attribute_prototype_id = if AttributePrototypeId::NONE == prototype_bag.id {
            create_new_attribute_prototype(ctx, &prototype_bag, func.id, key.clone()).await?
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
            remove_or_reset_attribute_prototype(
                ctx,
                attribute_prototype_id,
                attempt_to_reset_prototype,
            )
            .await?;
        }
    }

    // Use the "unset" response type if all bindings have been removed.
    if id_set.is_empty() {
        computed_backend_response_type = FuncBackendResponseType::Unset;
    }

    Ok(computed_backend_response_type)
}

// By default, remove the attribute prototype. If the user wishes to reset the prototype, they can,
// but only if the prototype is for an attribute value (i.e. if it is a component-specific
// prototype). If the prototype cannot be reset, it will be removed.
async fn remove_or_reset_attribute_prototype(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
    attempt_to_reset_prototype: bool,
) -> FuncAuthoringResult<()> {
    if attempt_to_reset_prototype {
        if let Some(attribute_value_id) =
            AttributePrototype::attribute_value_id(ctx, attribute_prototype_id).await?
        {
            AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
            return Ok(());
        }
    }
    AttributePrototype::remove(ctx, attribute_prototype_id).await?;
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
    key: Option<String>,
) -> FuncAuthoringResult<AttributePrototypeId> {
    let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;

    // TODO(nick): just destroy and burn nilId to the ground. We need to use the "id!" macro instead
    // of the "pk!" macro and be done with it.
    let component_id_cannot_be_nil_id = match prototype_bag.component_id {
        None | Some(ComponentId::NONE) => None,
        Some(component_id) => Some(component_id),
    };

    let mut affected_attribute_value_ids = Vec::new();

    if let Some(prop_id) = prototype_bag.prop_id {
        if let Some(component_id) = component_id_cannot_be_nil_id {
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
                EdgeWeightKind::Prototype(key),
            )
            .await?;
        }
    } else if let Some(output_socket_id) = prototype_bag.output_socket_id {
        if let Some(component_id) = component_id_cannot_be_nil_id {
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
                EdgeWeightKind::Prototype(key),
            )
            .await?;
        }
    } else {
        return Err(FuncAuthoringError::NoOutputLocationGiven(func_id));
    }

    if !affected_attribute_value_ids.is_empty() {
        ctx.add_dependent_values_and_enqueue(affected_attribute_value_ids)
            .await?;
    }

    Ok(attribute_prototype.id)
}
