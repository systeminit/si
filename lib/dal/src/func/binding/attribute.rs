use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ComponentId,
    InputSocketId,
    OutputSocketId,
    PropId,
    SchemaVariantId,
};
use telemetry::prelude::*;

use super::{
    AttributeArgumentBinding,
    AttributeFuncArgumentSource,
    AttributeFuncDestination,
    EventualParent,
    FuncBinding,
    FuncBindingError,
    FuncBindingResult,
};
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    AttributeValue,
    Component,
    DalContext,
    EdgeWeightKind,
    Func,
    FuncBackendKind,
    FuncId,
    OutputSocket,
    Prop,
    WorkspaceSnapshotError,
    attribute::prototype::{
        AttributePrototypeEventualParent,
        argument::AttributePrototypeArgument,
    },
    func::{
        FuncKind,
        argument::{
            FuncArgument,
            FuncArgumentError,
        },
        intrinsics::IntrinsicFunc,
    },
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
};

/// Contains the error scenarios for malformed input when creating or mutating attribute func bindings.
#[remain::sorted]
#[derive(Debug, Clone)]
pub enum AttributeBindingMalformedInput {
    /// The [`Component`]'s [`SchemaVariant`](crate::SchemaVariant) does not match the
    /// [`SchemaVariant`](crate::SchemaVariant) provided.
    EventualParentComponentNotFromSchemaVariant(ComponentId, SchemaVariantId),
    /// When assembling an input location, all options were provided. We only want one.
    InputLocationAllOptionsProvided(PropId, InputSocketId, serde_json::Value),
    /// When assembling an input location, both an [`InputSocketId`](crate::InputSocket) and a raw, static argument
    /// value were provided. We only want one option to be provided.
    InputLocationBothInputSocketAndStaticArgumentValueProvided(InputSocketId, serde_json::Value),
    /// When assembling an input location, both a [`PropId`](crate::Prop) and an [`InputSocketId`](crate::InputSocket)
    /// were provided. We only want one option to be provided.
    InputLocationBothPropAndInputSocketProvided(PropId, InputSocketId),
    /// When assembling an input location, both an [`PropId`](crate::Prop) and a raw, static argument
    /// value were provided. We only want one option to be provided.
    InputLocationBothPropAndStaticArgumentValueProvided(PropId, serde_json::Value),
    /// When assembling an input location, no option was provided. We want one option to be provided.
    InputLocationNoOptionProvided,
    /// When assembling an output location, both a [`PropId`](crate::Prop) and an [`OutputSocketId`](crate::OutputSocket)
    /// were provided. We only want one option to be provided.
    OutputLocationBothPropAndOutputSocketProvided(PropId, OutputSocketId),
    /// When assembling an output location, no option was provided. We want one option to be provided.
    OutputLocationNoOptionProvided,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributeBinding {
    // unique ids
    pub func_id: FuncId,
    pub attribute_prototype_id: AttributePrototypeId,
    // things needed for create
    pub eventual_parent: EventualParent,

    // things that can be updated
    pub output_location: AttributeFuncDestination,
    pub argument_bindings: Vec<AttributeArgumentBinding>,
}

impl AttributeBinding {
    pub async fn find_eventual_parent(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        let eventual_parent =
            AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
        let parent = match eventual_parent {
            AttributePrototypeEventualParent::Component(component_id, _) => {
                EventualParent::Component(component_id)
            }
            AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                schema_variant_id,
                _,
            ) => EventualParent::SchemaVariant(schema_variant_id),

            AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                schema_variant_id,
                _,
            ) => EventualParent::SchemaVariant(schema_variant_id),
            AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, _) => {
                EventualParent::SchemaVariant(schema_variant_id)
            }
        };
        Ok(parent)
    }

    pub(crate) async fn find_output_location(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<AttributeFuncDestination> {
        let eventual_parent =
            AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
        let output_location = match eventual_parent {
            AttributePrototypeEventualParent::Component(_, attribute_value_id) => {
                let prop_id = AttributeValue::prop_id(ctx, attribute_value_id).await?;
                AttributeFuncDestination::Prop(prop_id)
            }
            AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                _,
                output_socket_id,
            ) => AttributeFuncDestination::OutputSocket(output_socket_id),
            AttributePrototypeEventualParent::SchemaVariantFromProp(_, prop_id) => {
                AttributeFuncDestination::Prop(prop_id)
            }
            AttributePrototypeEventualParent::SchemaVariantFromInputSocket(_, input_socket_id) => {
                AttributeFuncDestination::InputSocket(input_socket_id)
            }
        };
        Ok(output_location)
    }

    pub async fn assemble_eventual_parent(
        ctx: &DalContext,
        component_id: Option<si_events::ComponentId>,
        schema_variant_id: Option<si_events::SchemaVariantId>,
    ) -> FuncBindingResult<Option<EventualParent>> {
        let eventual_parent = match (component_id, schema_variant_id) {
            (None, None) => None,
            (None, Some(schema_variant)) => Some(EventualParent::SchemaVariant(schema_variant)),
            (Some(component_id), None) => Some(EventualParent::Component(component_id)),
            (Some(component_id), Some(schema_variant_id)) => {
                if Component::schema_variant_id(ctx, component_id).await? == schema_variant_id {
                    Some(EventualParent::SchemaVariant(schema_variant_id))
                } else {
                    return Err(FuncBindingError::MalformedInput(
                        AttributeBindingMalformedInput::EventualParentComponentNotFromSchemaVariant(
                            component_id,
                            schema_variant_id,
                        ),
                    ));
                }
            }
        };
        Ok(eventual_parent)
    }

    pub fn assemble_attribute_input_location(
        prop_id: Option<si_events::PropId>,
        input_socket_id: Option<si_events::InputSocketId>,
        static_argument_value: Option<serde_json::Value>,
    ) -> FuncBindingResult<AttributeFuncArgumentSource> {
        match (prop_id, input_socket_id, static_argument_value) {
            (Some(prop_id), None, None) => Ok(AttributeFuncArgumentSource::Prop(prop_id)),
            (None, Some(input_socket_id), None) => Ok(AttributeFuncArgumentSource::InputSocket(
                input_socket_id,
            )),
            (None, None, Some(static_argument_value)) => Ok(
                AttributeFuncArgumentSource::StaticArgument(static_argument_value),
            ),
            (Some(prop_id), Some(input_socket_id), Some(static_argument_value)) => {
                Err(FuncBindingError::MalformedInput(
                    AttributeBindingMalformedInput::InputLocationAllOptionsProvided(
                        prop_id,
                        input_socket_id,
                        static_argument_value,
                    ),
                ))
            }
            (Some(prop_id), Some(input_socket_id), None) => Err(FuncBindingError::MalformedInput(
                AttributeBindingMalformedInput::InputLocationBothPropAndInputSocketProvided(
                    prop_id,
                    input_socket_id,
                ),
            )),
            (Some(prop_id), None, Some(static_argument_value)) => {
                Err(FuncBindingError::MalformedInput(
                    AttributeBindingMalformedInput::InputLocationBothPropAndStaticArgumentValueProvided(prop_id, static_argument_value),
                ))
            }
            (None, Some(input_socket_id), Some(static_argument_value)) => {
                Err(FuncBindingError::MalformedInput(
                    AttributeBindingMalformedInput::InputLocationBothInputSocketAndStaticArgumentValueProvided(input_socket_id, static_argument_value),
                ))
            }
            (None, None, None) => Err(FuncBindingError::MalformedInput(
                AttributeBindingMalformedInput::InputLocationNoOptionProvided,
            )),
        }
    }

    pub fn assemble_attribute_output_location(
        prop_id: Option<si_events::PropId>,
        output_socket_id: Option<si_events::OutputSocketId>,
    ) -> FuncBindingResult<AttributeFuncDestination> {
        match (prop_id, output_socket_id) {
            (Some(prop_id), None) => Ok(AttributeFuncDestination::Prop(prop_id)),
            (None, Some(output_socket_id)) => {
                Ok(AttributeFuncDestination::OutputSocket(output_socket_id))
            }
            (Some(prop_id), Some(output_socket_id)) => Err(FuncBindingError::MalformedInput(
                AttributeBindingMalformedInput::OutputLocationBothPropAndOutputSocketProvided(
                    prop_id,
                    output_socket_id,
                ),
            )),
            (None, None) => Err(FuncBindingError::MalformedInput(
                AttributeBindingMalformedInput::OutputLocationNoOptionProvided,
            )),
        }
    }

    /// assemble bindings for an [`IntrinsicFunc`]
    /// This filters out Component specific bindings and bindings for input sockets
    /// as we don't want users to be able to set prototypes for input sockets and we're
    /// not supported component specific bindings just yet
    pub async fn assemble_intrinsic_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut bindings = vec![];
        let intrinsic_func_kind: IntrinsicFunc =
            Func::get_intrinsic_kind_by_id_or_error(ctx, func_id).await?;

        for attribute_prototype_id in AttributePrototype::list_ids_for_func_id(ctx, func_id).await?
        {
            let eventual_parent = Self::find_eventual_parent(ctx, attribute_prototype_id).await?;
            // skip this binding if it's for a component (until we support component specific bindings)
            if let EventualParent::Component(component_id) = eventual_parent {
                trace!(
                    "skipping component {} for intrinsic {}",
                    component_id, intrinsic_func_kind
                );
                continue;
            }
            let output_location = Self::find_output_location(ctx, attribute_prototype_id).await?;
            // skip this binding if it's for an input socket as we don't let users change bindings for input sockets
            if let AttributeFuncDestination::InputSocket(input_socket_id) = output_location {
                trace!(
                    "skipping input socket {} for intrinsic {}",
                    input_socket_id, intrinsic_func_kind
                );
                continue;
            }
            let attribute_prototype_argument_ids =
                AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
                    .await?;

            let mut argument_bindings = Vec::with_capacity(attribute_prototype_argument_ids.len());
            for attribute_prototype_argument_id in attribute_prototype_argument_ids {
                argument_bindings.push(
                    AttributeArgumentBinding::assemble(ctx, attribute_prototype_argument_id)
                        .await?,
                );
            }
            bindings.push(FuncBinding::Attribute(AttributeBinding {
                func_id,
                attribute_prototype_id,
                eventual_parent,
                output_location,
                argument_bindings,
            }));
        }
        Ok(bindings)
    }

    pub async fn assemble_attribute_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut bindings = vec![];

        // Note(victor): AttributeArgumentBinding::assemble, fails for ValueSubscription value source.
        // Since this method does not care for these func bindings, we'll just return an empty list here
        if Func::get_by_id(ctx, func_id).await?.is_transformation {
            return Ok(bindings);
        }

        for attribute_prototype_id in AttributePrototype::list_ids_for_func_id(ctx, func_id).await?
        {
            let eventual_parent = Self::find_eventual_parent(ctx, attribute_prototype_id).await?;
            let output_location = Self::find_output_location(ctx, attribute_prototype_id).await?;
            let attribute_prototype_argument_ids =
                AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
                    .await?;

            let mut argument_bindings = Vec::with_capacity(attribute_prototype_argument_ids.len());
            for attribute_prototype_argument_id in attribute_prototype_argument_ids {
                argument_bindings.push(
                    AttributeArgumentBinding::assemble(ctx, attribute_prototype_argument_id)
                        .await?,
                );
            }
            bindings.push(FuncBinding::Attribute(AttributeBinding {
                func_id,
                attribute_prototype_id,
                eventual_parent,
                output_location,
                argument_bindings,
            }));
        }
        Ok(bindings)
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.attribute.upsert_attribute_binding"
    )]
    /// For a given [`AttributeFuncOutputLocation`], remove the existing [`AttributePrototype`]
    /// and arguments, then create a new one in it's place, with new arguments according to the
    /// [`AttributeArgumentBinding`]s
    /// Collect impacted AttributeValues and enqueue them for DependentValuesUpdate
    /// so the functions run upon being attached.
    /// Returns an error if we're trying to upsert an attribute binding for a locked [`SchemaVariant`]
    /// This is also used for Intrinsics, and we return an error if incorrect config values are passed in
    pub async fn upsert_attribute_binding(
        ctx: &DalContext,
        func_id: FuncId,
        eventual_parent: Option<EventualParent>,
        output_location: AttributeFuncDestination,
        prototype_arguments: Vec<AttributeArgumentBinding>,
    ) -> FuncBindingResult<(AttributePrototype, Option<FuncId>)> {
        let func = Func::get_by_id(ctx, func_id).await?;

        let needs_validate_intrinsic_input = match func.kind {
            FuncKind::Attribute => false,
            FuncKind::Intrinsic => true,
            kind => return Err(FuncBindingError::UnexpectedFuncKind(kind)),
        };

        // if a parent was specified, use it. otherwise find the schema variant
        // for the output location
        let eventual_parent = match eventual_parent {
            Some(eventual) => eventual,
            None => EventualParent::SchemaVariant(output_location.find_schema_variant(ctx).await?),
        };
        // return an error if the parent is a schema variant and it's locked
        eventual_parent.error_if_locked(ctx).await?;

        if needs_validate_intrinsic_input {
            validate_intrinsic_inputs(
                ctx,
                func_id,
                eventual_parent,
                output_location,
                prototype_arguments.clone(),
            )
            .await?;
        }

        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;
        let attribute_prototype_id = attribute_prototype.id;

        // cache attribute values that need to be updated from their prototype func after
        // we update the prototype
        let mut attribute_values_to_update = vec![];
        // need to track if the func bound has changed so we can fire the necessary events for the front end to handle the switch
        let mut maybe_existing_func_id: Option<FuncId> = None;
        match output_location {
            AttributeFuncDestination::Prop(prop_id) => {
                match eventual_parent {
                    EventualParent::SchemaVariant(_) => {
                        if let Some(existing_prototype_id) =
                            AttributePrototype::find_for_prop(ctx, prop_id, &None).await?
                        {
                            // let's see what the existing func is
                            let existing_func_id =
                                AttributePrototype::func_id(ctx, existing_prototype_id).await?;
                            maybe_existing_func_id = Some(existing_func_id);
                            // if we're setting this to unset, need to also clear any existing attribute values
                            if func.backend_kind == FuncBackendKind::Unset {
                                let attribute_value_ids = AttributePrototype::attribute_value_ids(
                                    ctx,
                                    existing_prototype_id,
                                )
                                .await?;
                                attribute_values_to_update.extend(attribute_value_ids);
                            }

                            // remove existing attribute prototype and arguments before we add the
                            // edge to the new one

                            Self::delete_attribute_prototype_and_args(ctx, existing_prototype_id)
                                .await?;
                        }
                        Prop::add_edge_to_attribute_prototype(
                            ctx,
                            prop_id,
                            attribute_prototype.id,
                            EdgeWeightKind::Prototype(None),
                        )
                        .await?;
                    }
                    EventualParent::Component(component_id) => {
                        let attribute_value_ids =
                            Component::attribute_values_for_prop_id(ctx, component_id, prop_id)
                                .await?;
                        // if we're setting this to unset, need to also clear any existing attribute values
                        if func.backend_kind == FuncBackendKind::Unset {
                            attribute_values_to_update.extend(attribute_value_ids.clone());
                        }

                        for attribute_value_id in attribute_value_ids {
                            AttributeValue::set_component_prototype_id(
                                ctx,
                                attribute_value_id,
                                attribute_prototype.id,
                                None,
                            )
                            .await?;
                        }
                    }
                }
            }
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                // remove existing attribute prototype and arguments
                match eventual_parent {
                    EventualParent::SchemaVariant(_) => {
                        if let Some(existing_prototype_id) =
                            AttributePrototype::find_for_output_socket(ctx, output_socket_id)
                                .await?
                        {
                            let existing_func_id =
                                AttributePrototype::func_id(ctx, existing_prototype_id).await?;
                            maybe_existing_func_id = Some(existing_func_id);
                            // if we're setting this to unset, need to also clear any existing attribute values
                            if func.backend_kind == FuncBackendKind::Unset {
                                let attribute_value_ids = AttributePrototype::attribute_value_ids(
                                    ctx,
                                    existing_prototype_id,
                                )
                                .await?;
                                attribute_values_to_update.extend(attribute_value_ids);
                            }

                            Self::delete_attribute_prototype_and_args(ctx, existing_prototype_id)
                                .await?;
                        }
                        OutputSocket::add_edge_to_attribute_prototype(
                            ctx,
                            output_socket_id,
                            attribute_prototype.id,
                            EdgeWeightKind::Prototype(None),
                        )
                        .await?;
                    }
                    EventualParent::Component(component_id) => {
                        let attribute_value_id = OutputSocket::component_attribute_value_id(
                            ctx,
                            output_socket_id,
                            component_id,
                        )
                        .await?;
                        // if we're setting this to unset, need to also clear any existing attribute values
                        if func.backend_kind == FuncBackendKind::Unset {
                            attribute_values_to_update.push(attribute_value_id);
                        }
                        AttributeValue::set_component_prototype_id(
                            ctx,
                            attribute_value_id,
                            attribute_prototype.id,
                            None,
                        )
                        .await?;
                    }
                }
            }
            // we don't let users configure this right now
            AttributeFuncDestination::InputSocket(_) => {
                return Err(FuncBindingError::InvalidAttributePrototypeDestination(
                    output_location,
                ));
            }
        }

        // if there are attribute values that need to be updated from prototype function - do it here!
        for attribute_value in attribute_values_to_update {
            AttributeValue::update_from_prototype_function(ctx, attribute_value).await?;
        }

        for arg in prototype_arguments {
            // Ensure a func argument exists for each input location, before creating new Attribute Prototype Arguments
            if let Err(err) = FuncArgument::get_by_id(ctx, arg.func_argument_id).await {
                match err {
                    FuncArgumentError::WorkspaceSnapshot(
                        WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                            WorkspaceSnapshotGraphError::NodeWithIdNotFound(raw_id),
                        ),
                    ) if raw_id == arg.func_argument_id.into() => {
                        continue;
                    }
                    err => return Err(err.into()),
                }
            }

            match arg.attribute_func_input_location {
                super::AttributeFuncArgumentSource::Prop(prop_id) => {
                    AttributePrototypeArgument::new(
                        ctx,
                        attribute_prototype_id,
                        arg.func_argument_id,
                        prop_id,
                    )
                    .await?;
                }
                super::AttributeFuncArgumentSource::InputSocket(input_socket_id) => {
                    AttributePrototypeArgument::new(
                        ctx,
                        attribute_prototype_id,
                        arg.func_argument_id,
                        input_socket_id,
                    )
                    .await?;
                }
                // note: this isn't in use yet, but is ready for when we enable users to set default values via the UI
                super::AttributeFuncArgumentSource::StaticArgument(value) => {
                    AttributePrototypeArgument::new_static_value(
                        ctx,
                        attribute_prototype_id,
                        arg.func_argument_id,
                        value,
                    )
                    .await?;
                }
                // we do not allow users to manually set these as inputs right now
                super::AttributeFuncArgumentSource::Secret(secret_id) => {
                    return Err(FuncBindingError::InvalidAttributePrototypeArgumentSource(
                        AttributeFuncArgumentSource::Secret(secret_id),
                    ));
                }
                super::AttributeFuncArgumentSource::OutputSocket(output_socket_id) => {
                    return Err(FuncBindingError::InvalidAttributePrototypeArgumentSource(
                        AttributeFuncArgumentSource::OutputSocket(output_socket_id),
                    ));
                }
            };
        }
        // enqueue dvu for impacted attribute values
        Self::enqueue_dvu_for_impacted_values(ctx, attribute_prototype_id).await?;
        Ok((attribute_prototype, maybe_existing_func_id))
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.attribute.update_attribute_binding_arguments"
    )]
    /// For a given [`AttributePrototypeId`], remove the existing [`AttributePrototype`]
    /// and arguments, then re-create them for the new inputs.
    pub async fn update_attribute_binding_arguments(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
        prototype_arguments: Vec<AttributeArgumentBinding>,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't update binding args if the parent is locked
        let eventual_parent = Self::find_eventual_parent(ctx, attribute_prototype_id).await?;
        eventual_parent.error_if_locked(ctx).await?;

        let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
        // if this func is intrinsic, make sure everything looks good
        if (Func::get_intrinsic_kind_by_id(ctx, func_id).await?).is_some() {
            let output_location = Self::find_output_location(ctx, attribute_prototype_id).await?;
            validate_intrinsic_inputs(
                ctx,
                func_id,
                eventual_parent,
                output_location,
                prototype_arguments.clone(),
            )
            .await?;
        };
        //remove existing arguments first
        Self::delete_attribute_prototype_args(ctx, attribute_prototype_id).await?;

        // recreate them
        for arg in prototype_arguments {
            // Ensure the func argument exists before continuing. By continuing, we will not add the
            // attribute prototype to the id set and will be deleted.
            if let Err(err) = FuncArgument::get_by_id(ctx, arg.func_argument_id).await {
                match err {
                    FuncArgumentError::WorkspaceSnapshot(
                        WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                            WorkspaceSnapshotGraphError::NodeWithIdNotFound(raw_id),
                        ),
                    ) if raw_id == arg.func_argument_id.into() => continue,
                    err => return Err(err.into()),
                }
            }

            match arg.attribute_func_input_location {
                super::AttributeFuncArgumentSource::Prop(prop_id) => {
                    AttributePrototypeArgument::new(
                        ctx,
                        attribute_prototype_id,
                        arg.func_argument_id,
                        prop_id,
                    )
                    .await?;
                }
                super::AttributeFuncArgumentSource::InputSocket(input_socket_id) => {
                    AttributePrototypeArgument::new(
                        ctx,
                        attribute_prototype_id,
                        arg.func_argument_id,
                        input_socket_id,
                    )
                    .await?;
                }
                // note: this isn't in use yet, but is ready for when we enable users to set default values via the UI
                super::AttributeFuncArgumentSource::StaticArgument(value) => {
                    AttributePrototypeArgument::new_static_value(
                        ctx,
                        attribute_prototype_id,
                        arg.func_argument_id,
                        value,
                    )
                    .await?;
                }
                // we do not allow users to manually set these as inputs right now
                super::AttributeFuncArgumentSource::Secret(secret_id) => {
                    return Err(FuncBindingError::InvalidAttributePrototypeArgumentSource(
                        AttributeFuncArgumentSource::Secret(secret_id),
                    ));
                }
                super::AttributeFuncArgumentSource::OutputSocket(output_socket_id) => {
                    return Err(FuncBindingError::InvalidAttributePrototypeArgumentSource(
                        AttributeFuncArgumentSource::OutputSocket(output_socket_id),
                    ));
                }
            };
        }
        // enqueue dvu for impacted attribute values
        Self::enqueue_dvu_for_impacted_values(ctx, attribute_prototype_id).await?;
        FuncBinding::for_func_id(ctx, func_id).await
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.attribute.reset_attribute_binding"
    )]
    /// Deletes the current [`AttributePrototype`] node and all associated [`AttributePrototypeArgument`]s
    pub(crate) async fn delete_attribute_prototype_and_args(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<()> {
        // don't update binding args if the parent is locked
        let eventual_parent = Self::find_eventual_parent(ctx, attribute_prototype_id).await?;
        eventual_parent.error_if_locked(ctx).await?;

        Self::delete_attribute_prototype_args(ctx, attribute_prototype_id).await?;
        // should we fire a WsEvent here in case we just dropped an existing user authored
        // attribute func?
        AttributePrototype::remove(ctx, attribute_prototype_id).await?;
        Ok(())
    }
    async fn delete_attribute_prototype_args(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<()> {
        let current_attribute_prototype_arguments =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;
        for apa in current_attribute_prototype_arguments {
            AttributePrototypeArgument::remove(ctx, apa).await?;
        }
        Ok(())
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.attribute.reset_attribute_binding"
    )]
    /// For a given [`AttributePrototypeId`], remove the existing [`AttributePrototype`] and [`AttributePrototypeArgument`]s
    /// For a [`Component`], we'll reset the prototype to what is defined for the [`SchemaVariant`], and for now, reset the
    /// [`SchemaVariant`]'s prototype to be the si:Unset. When the user regenerates the schema, we'll re-apply whatever has
    /// been configured in the Schema Def function. This is a hold over until we remove this behavior from being configured in the
    /// definition func and enable users to set intrinsic funcs via the UI.
    pub async fn reset_attribute_binding(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        // don't update binding args if the parent is locked
        let eventual_parent = Self::find_eventual_parent(ctx, attribute_prototype_id).await?;
        eventual_parent.error_if_locked(ctx).await?;

        if let Some(attribute_value_id) =
            AttributePrototype::attribute_value_id(ctx, attribute_prototype_id).await?
        {
            AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
        } else {
            // let's set the prototype to unset so that when we regenerate,
            // the socket or prop's prototype can get reset to the value from (if that is where it was coming from)
            // or the default value as defined in the schema variant def

            let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
            AttributePrototype::update_func_by_id(ctx, attribute_prototype_id, unset_func_id)
                .await?;

            // loop through and delete all existing attribute prototype arguments
            let current_attribute_prototype_arguments =
                AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
                    .await?;
            for apa in current_attribute_prototype_arguments {
                AttributePrototypeArgument::remove(ctx, apa).await?;
            }
        }
        // enqueue dvu for impacted attribute values
        Self::enqueue_dvu_for_impacted_values(ctx, attribute_prototype_id).await?;
        Ok(eventual_parent)
    }

    /// For a given [`AttributePrototypeId`], find all [`AttributeValue`]s that use it, and enqueue them for dependent
    /// values update so they update on commit!
    pub async fn enqueue_dvu_for_impacted_values(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<()> {
        // get the impacted attribute values
        let impacted_avs =
            AttributePrototype::attribute_value_ids(ctx, attribute_prototype_id).await?;

        // enqueue them for DVU
        if !impacted_avs.is_empty() {
            ctx.add_dependent_values_and_enqueue(impacted_avs).await?;
        }
        Ok(())
    }

    pub(crate) async fn compile_attribute_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<String> {
        let mut input_ts_types = "type Input = {\n".to_string();

        let mut output_ts_types = vec![];
        let mut argument_types = HashMap::new();
        let bindings = Self::assemble_attribute_bindings(ctx, func_id).await?;
        for binding in bindings {
            if let FuncBinding::Attribute(attribute) = binding {
                for arg in attribute.argument_bindings {
                    if let AttributeFuncArgumentSource::Prop(prop_id) =
                        arg.attribute_func_input_location
                    {
                        let ts_type = Prop::ts_type(ctx, prop_id).await?;

                        if let std::collections::hash_map::Entry::Vacant(e) =
                            argument_types.entry(arg.func_argument_id)
                        {
                            e.insert(vec![ts_type]);
                        } else if let Some(ts_types_for_arg) =
                            argument_types.get_mut(&arg.func_argument_id)
                        {
                            if !ts_types_for_arg.contains(&ts_type) {
                                ts_types_for_arg.push(ts_type)
                            }
                        }
                    }
                    let output_type = if let AttributeFuncDestination::Prop(output_prop_id) =
                        attribute.output_location
                    {
                        Prop::ts_type(ctx, output_prop_id).await?
                    } else {
                        "any".to_string()
                    };
                    if !output_ts_types.contains(&output_type) {
                        output_ts_types.push(output_type);
                    }
                }
            }
        }

        for (arg_id, ts_types) in argument_types.iter() {
            let func_arg = FuncArgument::get_by_id(ctx, *arg_id).await?;
            let arg_name = func_arg.name;
            input_ts_types
                .push_str(format!("{}?: {} | null;\n", arg_name, ts_types.join(" | ")).as_str());
        }
        input_ts_types.push_str("};");

        let output_ts = format!("type Output = {};", output_ts_types.join(" | "));

        Ok(format!("{input_ts_types}\n{output_ts}"))
    }

    /// Take the existing [`AttributeBinding`] and recreate it for the new [`Func`]
    pub(crate) async fn port_binding_to_new_func(
        &self,
        ctx: &DalContext,
        new_func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // get the updated AttributeArgumentBindings (pointing at the new func arg ids)
        let mut args_to_update = vec![];

        let new_args = FuncArgument::list_for_func(ctx, new_func_id).await?;
        for arg in &self.argument_bindings {
            // get the func arg mapping in the new func
            let old_arg = FuncArgument::get_name_by_id(ctx, arg.func_argument_id).await?;
            if let Some(new_arg) = new_args.clone().into_iter().find(|arg| arg.name == old_arg) {
                args_to_update.push(AttributeArgumentBinding {
                    func_argument_id: new_arg.id,
                    attribute_prototype_argument_id: None,
                    attribute_func_input_location: arg.attribute_func_input_location.clone(),
                })
            } else {
                return Err(FuncBindingError::FuncArgumentMissing(
                    arg.func_argument_id,
                    old_arg,
                ));
            }
        }
        // delete and recreate attribute prototype and args

        Self::upsert_attribute_binding(
            ctx,
            new_func_id,
            None,
            self.output_location,
            args_to_update,
        )
        .await?;

        FuncBinding::for_func_id(ctx, new_func_id).await
    }
}

async fn validate_intrinsic_inputs(
    ctx: &DalContext,
    func_id: FuncId,
    eventual_parent: EventualParent,
    output_location: AttributeFuncDestination,
    prototype_arguments: Vec<AttributeArgumentBinding>,
) -> FuncBindingResult<()> {
    let intrinsic_kind = Func::get_intrinsic_kind_by_id_or_error(ctx, func_id).await?;
    if let EventualParent::Component(component_id) = eventual_parent {
        return Err(FuncBindingError::CannotSetIntrinsicForComponent(
            component_id,
        ));
    }
    match intrinsic_kind {
        IntrinsicFunc::Identity
        | IntrinsicFunc::NormalizeToArray
        | IntrinsicFunc::ResourcePayloadToValue => {
            // for now we only support configuring one input location at a time
            if prototype_arguments.len() > 1 {
                return Err(FuncBindingError::InvalidIntrinsicBinding);
            }
            match output_location {
                // props can only take input from other props and input sockets
                AttributeFuncDestination::Prop(_) => {
                    let mut maybe_invalid_inputs = prototype_arguments.clone();
                    maybe_invalid_inputs.retain(|arg| match arg.attribute_func_input_location {
                        AttributeFuncArgumentSource::Prop(_) => false,
                        AttributeFuncArgumentSource::InputSocket(_) => false,
                        AttributeFuncArgumentSource::StaticArgument(_) => true,
                        AttributeFuncArgumentSource::OutputSocket(_) => true,
                        AttributeFuncArgumentSource::Secret(_) => true,
                    });
                    if !maybe_invalid_inputs.is_empty() {
                        return Err(FuncBindingError::InvalidIntrinsicBinding);
                    }
                }
                // output sockets can take input from props or input sockets
                AttributeFuncDestination::OutputSocket(_) => {
                    let mut maybe_invalid_inputs = prototype_arguments.clone();
                    maybe_invalid_inputs.retain(|arg| match arg.attribute_func_input_location {
                        AttributeFuncArgumentSource::Prop(_) => false,
                        AttributeFuncArgumentSource::InputSocket(_) => false,
                        AttributeFuncArgumentSource::StaticArgument(_) => true,
                        AttributeFuncArgumentSource::OutputSocket(_) => true,
                        AttributeFuncArgumentSource::Secret(_) => true,
                    });
                    if !maybe_invalid_inputs.is_empty() {
                        return Err(FuncBindingError::InvalidIntrinsicBinding);
                    }
                }
                // input sockets can't take input from anything this way
                AttributeFuncDestination::InputSocket(_) => {
                    return Err(FuncBindingError::InvalidAttributePrototypeDestination(
                        output_location,
                    ));
                }
            }
        }
        IntrinsicFunc::Unset => {
            // ensure no args are sent
            if !prototype_arguments.is_empty() {
                return Err(FuncBindingError::InvalidIntrinsicBinding);
            }
            match output_location {
                AttributeFuncDestination::Prop(_) | AttributeFuncDestination::OutputSocket(_) => {}
                AttributeFuncDestination::InputSocket(_) => {
                    return Err(FuncBindingError::InvalidIntrinsicBinding);
                }
            }
        }
        IntrinsicFunc::SetArray
        | IntrinsicFunc::SetBoolean
        | IntrinsicFunc::SetInteger
        | IntrinsicFunc::SetFloat
        | IntrinsicFunc::SetJson
        | IntrinsicFunc::SetMap
        | IntrinsicFunc::SetObject
        | IntrinsicFunc::SetString => {
            // ensure there's only one value
            if prototype_arguments.len() > 1 {
                return Err(FuncBindingError::InvalidIntrinsicBinding);
            }
        }
        IntrinsicFunc::Validation => return Err(FuncBindingError::InvalidIntrinsicBinding),
    };
    Ok(())
}
