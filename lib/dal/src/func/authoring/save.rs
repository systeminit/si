use std::collections::HashSet;
use telemetry::prelude::*;

use crate::action::prototype::{ActionKind, ActionPrototype};
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::associations::FuncAssociations;
use crate::func::authoring::{FuncAuthoringError, FuncAuthoringResult};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::{AttributePrototypeArgumentBag, AttributePrototypeBag, FuncKind};
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::{
    AttributePrototype, AttributePrototypeId, AttributeValue, Component, ComponentId, DalContext,
    EdgeWeightKind, Func, FuncBackendResponseType, FuncId, OutputSocket, Prop, SchemaVariant,
    SchemaVariantId, WorkspaceSnapshotError,
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
        // don't update attribute associations this way
        // attribute associations are updated through calling
        // create/remove/update attribute binding directly
        FuncKind::Attribute => Ok(()),
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
    kind: ActionKind,
    schema_variant_ids: Vec<SchemaVariantId>,
) -> FuncAuthoringResult<()> {
    let id_set: HashSet<SchemaVariantId> = HashSet::from_iter(schema_variant_ids.iter().copied());

    // Add the new action to schema variants who do not already have a prototype or re-create the
    // prototype if the kind has been mutated.
    for schema_variant_id in schema_variant_ids {
        let existing_action_prototypes =
            ActionPrototype::for_variant(ctx, schema_variant_id).await?;

        // Assume that the prototype needs to be created. Bail the moment that we know one already
        // exists the moment that we know that the kind has been mutated, which means we will need
        // to re-create the prototype with the new kind.
        let mut needs_creation = true;
        let mut outdated_action_prototype_id = None;
        for (existing_action_prototype_id, exiting_action_prototype_kind) in
            existing_action_prototypes.iter().map(|p| (p.id, p.kind))
        {
            let prototype_func_id =
                ActionPrototype::func_id(ctx, existing_action_prototype_id).await?;

            // Match found! We need to now decide if we need to re-create the prototype. If the user
            // is keeping the prototype kind the same, then we don't need to create a new prototype.
            // If the user wishes to mutate the kind, then we need to delete the existing prototype
            // and create a new one.
            if func.id == prototype_func_id {
                if kind == exiting_action_prototype_kind {
                    needs_creation = false;
                } else {
                    outdated_action_prototype_id = Some(existing_action_prototype_id);
                }
                break;
            }
        }

        // Any time that we need to create a new prototype, we need to first check that it will not
        // collide with an existing prototype using the same, non-manual kind that we provided.
        if needs_creation {
            if kind != ActionKind::Manual
                && existing_action_prototypes.iter().any(|p| p.kind == kind)
            {
                return Err(FuncAuthoringError::ActionKindAlreadyExists(
                    kind,
                    schema_variant_id,
                ));
            }

            // Remove the prototype that needs to be re-created, if necessary.
            if let Some(outdated_action_prototype_id) = outdated_action_prototype_id {
                ActionPrototype::remove(ctx, outdated_action_prototype_id).await?;
            }

            ActionPrototype::new(
                ctx,
                kind,
                func.name.to_owned(),
                func.description.to_owned(),
                schema_variant_id,
                func.id,
            )
            .await?;
        }
    }

    // Remove action prototypes using our func from schema variants that weren't seen.
    for action_prototype_id in ActionPrototype::list_for_func_id(ctx, func.id).await? {
        let schema_variant_id =
            ActionPrototype::schema_variant_id(ctx, action_prototype_id).await?;
        if !id_set.contains(&schema_variant_id) {
            ActionPrototype::remove(ctx, action_prototype_id).await?;
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
) -> FuncAuthoringResult<()> {
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
pub(crate) async fn remove_or_reset_attribute_prototype(
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
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;
    AttributePrototype::update_func_by_id(ctx, attribute_prototype_id, identity_func_id).await?;
    Ok(())
}

pub(crate) async fn save_attr_func_proto_arguments(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
    arguments: Vec<AttributePrototypeArgumentBag>,
) -> FuncAuthoringResult<()> {
    let mut id_set = HashSet::new();

    for arg in &arguments {
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

pub(crate) async fn create_new_attribute_prototype(
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
