use std::collections::HashMap;
use telemetry::prelude::*;

use crate::{
    attribute::prototype::{
        argument::AttributePrototypeArgument, AttributePrototypeEventualParent,
    },
    func::{
        argument::{FuncArgument, FuncArgumentError},
        intrinsics::IntrinsicFunc,
        FuncKind,
    },
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
    AttributePrototype, AttributePrototypeId, AttributeValue, Component, DalContext,
    EdgeWeightKind, Func, FuncId, OutputSocket, Prop, WorkspaceSnapshotError,
};

use super::{
    AttributeArgumentBinding, AttributeFuncArgumentSource, AttributeFuncDestination,
    EventualParent, FuncBinding, FuncBindings, FuncBindingsError, FuncBindingsResult,
};

pub struct AttributeBinding;

impl AttributeBinding {
    pub async fn find_eventual_parent(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingsResult<EventualParent> {
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
    ) -> FuncBindingsResult<AttributeFuncDestination> {
        let eventual_parent =
            AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
        let output_location = match eventual_parent {
            AttributePrototypeEventualParent::Component(_, attribute_value_id) => {
                let prop_id =
                    AttributeValue::prop_id_for_id_or_error(ctx, attribute_value_id).await?;
                AttributeFuncDestination::Prop(prop_id)
            }
            AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                _,
                output_socket_id,
            ) => AttributeFuncDestination::OutputSocket(output_socket_id),
            AttributePrototypeEventualParent::SchemaVariantFromProp(_, prop_id) => {
                AttributeFuncDestination::Prop(prop_id)
            }
            AttributePrototypeEventualParent::SchemaVariantFromInputSocket(_, _) => {
                return Err(FuncBindingsError::MalformedInput("()".to_owned()));
            }
        };
        Ok(output_location)
    }

    pub async fn assemble_eventual_parent(
        ctx: &DalContext,
        component_id: Option<si_events::ComponentId>,
        schema_variant_id: Option<si_events::SchemaVariantId>,
    ) -> FuncBindingsResult<Option<EventualParent>> {
        let eventual_parent = match (component_id, schema_variant_id) {
            (None, None) => None,
            (None, Some(schema_variant)) => {
                Some(EventualParent::SchemaVariant(schema_variant.into()))
            }
            (Some(component_id), None) => Some(EventualParent::Component(component_id.into())),
            (Some(component_id), Some(schema_variant)) => {
                if Component::schema_variant_id(ctx, component_id.into()).await?
                    == schema_variant.into()
                {
                    Some(EventualParent::SchemaVariant(schema_variant.into()))
                } else {
                    return Err(FuncBindingsError::MalformedInput(
                        "component and schema variant mismatch".to_owned(),
                    ));
                }
            }
        };
        Ok(eventual_parent)
    }
    pub fn assemble_attribute_output_location(
        prop_id: Option<si_events::PropId>,
        output_socket_id: Option<si_events::OutputSocketId>,
    ) -> FuncBindingsResult<AttributeFuncDestination> {
        let output_location = match (prop_id, output_socket_id) {
            (None, Some(output_socket_id)) => {
                AttributeFuncDestination::OutputSocket(output_socket_id.into())
            }

            (Some(prop_id), None) => AttributeFuncDestination::Prop(prop_id.into()),
            _ => {
                return Err(FuncBindingsError::MalformedInput(
                    "cannot set more than one output location".to_owned(),
                ))
            }
        };
        Ok(output_location)
    }

    pub(crate) async fn assemble_attribute_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<Vec<FuncBinding>> {
        let mut bindings = vec![];
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

            bindings.push(FuncBinding::Attribute {
                func_id,
                attribute_prototype_id,
                eventual_parent,
                output_location,
                argument_bindings,
            });
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
    /// Collect impacted AttributeValues along the way and enqueue them for DependentValuesUpdate
    /// so the functions run upon being attached.
    pub async fn upsert_attribute_binding(
        ctx: &DalContext,
        func_id: FuncId,
        eventual_parent: Option<EventualParent>,
        output_location: AttributeFuncDestination,
        prototype_arguments: Vec<AttributeArgumentBinding>,
    ) -> FuncBindingsResult<FuncBindings> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        if func.kind != FuncKind::Attribute {
            return Err(FuncBindingsError::UnexpectedFuncKind(func.kind));
        }
        let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;
        let attribute_prototype_id = attribute_prototype.id;
        let mut affected_attribute_value_ids = vec![];
        // if a parent was specified, use it. otherwise find the schema variant
        // for the output location
        let eventual_parent = match eventual_parent {
            Some(eventual) => eventual,
            None => EventualParent::SchemaVariant(output_location.find_schema_variant(ctx).await?),
        };
        match output_location {
            AttributeFuncDestination::Prop(prop_id) => {
                match eventual_parent {
                    EventualParent::SchemaVariant(_) => {
                        if let Some(existing_prototype_id) =
                            AttributePrototype::find_for_prop(ctx, prop_id, &None).await?
                        {
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
                            Prop::attribute_values_for_prop_id(ctx, prop_id).await?;

                        for attribute_value_id in attribute_value_ids {
                            if component_id
                                == AttributeValue::component_id(ctx, attribute_value_id).await?
                            {
                                AttributeValue::set_component_prototype_id(
                                    ctx,
                                    attribute_value_id,
                                    attribute_prototype.id,
                                    None,
                                )
                                .await?;
                                affected_attribute_value_ids.push(attribute_value_id);
                            }
                        }
                    }
                }
            }
            AttributeFuncDestination::OutputSocket(output_socket_id) => {
                // remove existing attribute prototype and arguments
                match eventual_parent {
                    EventualParent::SchemaVariant(_) => {
                        if let Some(existing_proto) =
                            AttributePrototype::find_for_output_socket(ctx, output_socket_id)
                                .await?
                        {
                            Self::delete_attribute_prototype_and_args(ctx, existing_proto).await?;
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
                        let attribute_value_ids =
                            OutputSocket::attribute_values_for_output_socket_id(
                                ctx,
                                output_socket_id,
                            )
                            .await?;
                        for attribute_value_id in attribute_value_ids {
                            if component_id
                                == AttributeValue::component_id(ctx, attribute_value_id).await?
                            {
                                AttributeValue::set_component_prototype_id(
                                    ctx,
                                    attribute_value_id,
                                    attribute_prototype.id,
                                    None,
                                )
                                .await?;
                                affected_attribute_value_ids.push(attribute_value_id);
                            }
                        }
                    }
                }
            }
        }
        if !affected_attribute_value_ids.is_empty() {
            ctx.add_dependent_values_and_enqueue(affected_attribute_value_ids)
                .await?;
        }
        for arg in &prototype_arguments {
            // Ensure a func argument exists for each input location, before creating new Attribute Prototype Arguments
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

            let attribute_prototype_argument =
                AttributePrototypeArgument::new(ctx, attribute_prototype_id, arg.func_argument_id)
                    .await?;
            match &arg.attribute_func_input_location {
                super::AttributeFuncArgumentSource::Prop(prop_id) => {
                    attribute_prototype_argument
                        .set_value_from_prop_id(ctx, *prop_id)
                        .await?
                }
                super::AttributeFuncArgumentSource::InputSocket(input_socket_id) => {
                    attribute_prototype_argument
                        .set_value_from_input_socket_id(ctx, *input_socket_id)
                        .await?
                }
                // note: this isn't in use yet, but is ready for when we enable users to set default values via the UI
                super::AttributeFuncArgumentSource::StaticArgument(value) => {
                    attribute_prototype_argument
                        .set_value_from_static_value(
                            ctx,
                            serde_json::from_str::<serde_json::Value>(value.as_str())?,
                        )
                        .await?
                }
            };
        }
        let new_bindings = FuncBindings::from_func_id(ctx, func_id).await?;
        Ok(new_bindings)
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
    ) -> FuncBindingsResult<FuncBindings> {
        let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
        //remove existing arguments first
        Self::delete_attribute_prototype_args(ctx, attribute_prototype_id).await?;

        // recreate them
        for arg in &prototype_arguments {
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

            let attribute_prototype_argument =
                AttributePrototypeArgument::new(ctx, attribute_prototype_id, arg.func_argument_id)
                    .await?;
            match &arg.attribute_func_input_location {
                super::AttributeFuncArgumentSource::Prop(prop_id) => {
                    attribute_prototype_argument
                        .set_value_from_prop_id(ctx, *prop_id)
                        .await?
                }
                super::AttributeFuncArgumentSource::InputSocket(input_socket_id) => {
                    attribute_prototype_argument
                        .set_value_from_input_socket_id(ctx, *input_socket_id)
                        .await?
                }
                super::AttributeFuncArgumentSource::StaticArgument(value) => {
                    attribute_prototype_argument
                        .set_value_from_static_value(
                            ctx,
                            serde_json::from_str::<serde_json::Value>(value.as_str())?,
                        )
                        .await?
                }
            };
        }
        let new_bindings = FuncBindings::from_func_id(ctx, func_id).await?;
        Ok(new_bindings)
    }

    pub(crate) async fn delete_attribute_prototype_and_args(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingsResult<()> {
        Self::delete_attribute_prototype_args(ctx, attribute_prototype_id).await?;
        // should we fire a WsEvent here in case we just dropped an existing user authored
        // attribute func?
        AttributePrototype::remove(ctx, attribute_prototype_id).await?;
        Ok(())
    }
    async fn delete_attribute_prototype_args(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingsResult<()> {
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
    /// [`SchemaVariant`]'s prototype to be the Identity Func. When the user regenerates the schema, we'll re-apply whatever has
    /// been configured in the Schema Def function. This is a hold over until we remove this behavior from being configured in the
    /// definition func and enable users to set intrinsic funcs via the UI.
    pub async fn reset_attribute_binding(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingsResult<FuncBindings> {
        let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;

        if let Some(attribute_value_id) =
            AttributePrototype::attribute_value_id(ctx, attribute_prototype_id).await?
        {
            AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
        } else {
            // We're trying to reset the schema variant's prorotype,
            // so set this prototype to be identity and remove all existing arguments.
            // By setting to identity, this ensures that IF the user regenerates the schema variant def in the future,
            // we'll correctly reset the value sources based on what's in that code

            let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;
            AttributePrototype::update_func_by_id(ctx, attribute_prototype_id, identity_func_id)
                .await?;
            // loop through and delete all existing attribute prototype arguments
            let current_attribute_prototype_arguments =
                AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
                    .await?;
            for apa in current_attribute_prototype_arguments {
                AttributePrototypeArgument::remove(ctx, apa).await?;
            }
        }
        let new_binding = FuncBindings::from_func_id(ctx, func_id).await?;
        Ok(new_binding)
    }

    pub(crate) async fn compile_attribute_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<String> {
        let mut input_ts_types = "type Input = {\n".to_string();

        let mut output_ts_types = vec![];
        let mut argument_types = HashMap::new();
        let bindings = Self::assemble_attribute_bindings(ctx, func_id).await?;
        for binding in bindings {
            if let FuncBinding::Attribute {
                output_location,
                argument_bindings,
                ..
            } = binding
            {
                for arg in argument_bindings {
                    if let AttributeFuncArgumentSource::Prop(prop_id) =
                        arg.attribute_func_input_location
                    {
                        let prop = Prop::get_by_id_or_error(ctx, prop_id).await?;
                        let ts_type = prop.ts_type(ctx).await?;

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
                    let output_type =
                        if let AttributeFuncDestination::Prop(output_prop_id) = output_location {
                            Prop::get_by_id_or_error(ctx, output_prop_id)
                                .await?
                                .ts_type(ctx)
                                .await?
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
            let func_arg = FuncArgument::get_by_id_or_error(ctx, *arg_id).await?;
            let arg_name = func_arg.name;
            input_ts_types
                .push_str(format!("{}?: {} | null;\n", arg_name, ts_types.join(" | ")).as_str());
        }
        input_ts_types.push_str("};");

        let output_ts = format!("type Output = {};", output_ts_types.join(" | "));

        Ok(format!("{}\n{}", input_ts_types, output_ts))
    }
}
