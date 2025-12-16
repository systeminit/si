use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_types::LeafBindingPrototype;
use si_id::LeafPrototypeId;
use telemetry::prelude::*;

use super::{
    AttributeBinding,
    EventualParent,
    FuncBinding,
    FuncBindingResult,
};
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    AttributeValue,
    Component,
    DalContext,
    Func,
    FuncId,
    Prop,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    attribute::prototype::argument::AttributePrototypeArgument,
    func::{
        argument::FuncArgument,
        intrinsics::IntrinsicFunc,
        leaf::{
            LeafInput,
            LeafInputLocation,
            LeafKind,
        },
    },
    prop::PropPath,
    schema::leaf::LeafPrototype,
    workspace_snapshot::edge_weight::EdgeWeightKind,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LeafBinding {
    // unique ids
    pub func_id: FuncId,
    pub leaf_binding_prototype: LeafBindingPrototype,
    // things needed for create
    pub eventual_parent: EventualParent,
    // thing that can be updated
    pub inputs: Vec<LeafInputLocation>,
    // kind to differentiate if needed
    pub leaf_kind: LeafKind,
}

impl LeafBinding {
    pub async fn assemble_leaf_func_bindings(
        ctx: &DalContext,
        func_id: FuncId,
        leaf_kind: LeafKind,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut bindings = vec![];
        let leaf_prototype_ids = LeafPrototype::for_func(ctx, func_id)
            .await
            .map_err(Box::new)?;
        for leaf_proto in leaf_prototype_ids {
            let inputs = leaf_proto.leaf_inputs().collect();
            let schema_ids = LeafPrototype::schemas(ctx, leaf_proto.id())
                .await
                .map_err(Box::new)?;
            let leaf_binding = LeafBinding {
                func_id,
                leaf_binding_prototype: LeafBindingPrototype::Overlay(leaf_proto.id()),
                eventual_parent: EventualParent::Schemas(schema_ids),
                inputs,
                leaf_kind,
            };

            bindings.push(match leaf_kind {
                LeafKind::CodeGeneration => FuncBinding::CodeGeneration(leaf_binding),
                LeafKind::Qualification => FuncBinding::Qualification(leaf_binding),
            });
        }

        let attribute_prototype_ids =
            AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
        let inputs = Self::list_leaf_function_inputs(ctx, func_id).await?;

        for attribute_prototype_id in attribute_prototype_ids {
            let eventual_parent =
                AttributeBinding::find_eventual_parent(ctx, attribute_prototype_id).await?;

            if let EventualParent::Component(_) = eventual_parent {
                // skip components for now
                continue;
            }

            let leaf_binding = LeafBinding {
                func_id,
                leaf_binding_prototype: LeafBindingPrototype::Attribute(attribute_prototype_id),
                eventual_parent,
                inputs: inputs.clone(),
                leaf_kind,
            };

            let binding = match leaf_kind {
                LeafKind::CodeGeneration => FuncBinding::CodeGeneration(leaf_binding),
                LeafKind::Qualification => FuncBinding::Qualification(leaf_binding),
            };

            bindings.push(binding)
        }
        Ok(bindings)
    }

    async fn list_leaf_function_inputs(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<LeafInputLocation>> {
        Ok(FuncArgument::list_for_func(ctx, func_id)
            .await?
            .iter()
            .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(&arg.name))
            .collect())
    }

    /// Create an Attribute Prototype for the given [`LeafKind`], with the provided input locations.
    /// If no input locations are provided, default to [`LeafInputLocation::Domain`].
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.leaf.create_leaf_func_binding"
    )]
    pub async fn create_leaf_func_binding(
        ctx: &DalContext,
        func_id: FuncId,
        eventual_parent: EventualParent,
        leaf_kind: LeafKind,
        inputs: &[LeafInputLocation],
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't create binding if parent is locked
        eventual_parent.error_if_locked(ctx).await?;

        let func = Func::get_by_id(ctx, func_id).await?;
        match eventual_parent {
            EventualParent::SchemaVariant(schema_variant_id) => {
                let inputs = match inputs.is_empty() {
                    true => &[LeafInputLocation::Domain],
                    false => inputs,
                };
                SchemaVariant::upsert_leaf_function(
                    ctx,
                    schema_variant_id,
                    leaf_kind,
                    inputs,
                    &func,
                )
                .await?;
            }
            EventualParent::Component(_) => {
                //brit todo create this func
                // let attribute_prototype_id =
                //     Component::upsert_leaf_function(ctx, component_id, leaf_kind, inputs, &func).await?;
            }
            EventualParent::Schemas(schemas) => {
                let mut schema_iter = schemas.iter();
                let Some(first_schema_id) = schema_iter.next().copied() else {
                    return Err(super::FuncBindingError::NoSchemas);
                };

                let prototype =
                    LeafPrototype::new(ctx, first_schema_id, leaf_kind, inputs, func.id)
                        .await
                        .map_err(Box::new)?;

                for next_schema_id in schema_iter {
                    prototype
                        .attach_to_schema(ctx, *next_schema_id)
                        .await
                        .map_err(Box::new)?;
                }
            }
        }

        let new_bindings = FuncBinding::for_func_id(ctx, func_id).await?;
        Ok(new_bindings)
    }

    pub(crate) async fn port_binding_to_new_func(
        ctx: &DalContext,
        new_func_id: FuncId,
        leaf_binding_prototype: LeafBindingPrototype,
        leaf_kind: LeafKind,
        eventual_parent: EventualParent,
        inputs: &[LeafInputLocation],
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        match leaf_binding_prototype {
            LeafBindingPrototype::Attribute(attribute_prototype_id) => {
                LeafBinding::delete_leaf_func_binding(ctx, attribute_prototype_id).await?;
            }
            LeafBindingPrototype::Overlay(leaf_prototype_id) => {
                LeafBinding::delete_leaf_overlay_func_binding(ctx, leaf_prototype_id).await?;
            }
        }

        // create one for the new func_id
        LeafBinding::create_leaf_func_binding(ctx, new_func_id, eventual_parent, leaf_kind, inputs)
            .await?;
        FuncBinding::for_func_id(ctx, new_func_id).await
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.leaf.update_leaf_func_binding"
    )]
    pub async fn update_leaf_func_binding(
        ctx: &DalContext,
        leaf_binding_prototype: LeafBindingPrototype,
        input_locations: &[LeafInputLocation],
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let func_id = match leaf_binding_prototype {
            LeafBindingPrototype::Attribute(attribute_prototype_id) => {
                let eventual_parent =
                    AttributeBinding::find_eventual_parent(ctx, attribute_prototype_id).await?;
                eventual_parent.error_if_locked(ctx).await?;

                // find the prototype
                let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
                // update the input locations
                let mut existing_args = FuncArgument::list_for_func(ctx, func_id).await?;
                let mut inputs = vec![];
                for location in input_locations {
                    let arg_name = location.arg_name();
                    let arg_id = match existing_args
                        .iter()
                        .find(|arg| arg.name.as_str() == arg_name)
                    {
                        Some(existing_arg) => existing_arg.id,
                        None => {
                            let new_arg = FuncArgument::new(
                                ctx,
                                arg_name,
                                location.arg_kind(),
                                None,
                                func_id,
                            )
                            .await?;
                            new_arg.id
                        }
                    };

                    inputs.push(LeafInput {
                        location: *location,
                        func_argument_id: arg_id,
                    });

                    if let EventualParent::SchemaVariant(schema_variant_id) = eventual_parent {
                        SchemaVariant::upsert_leaf_function_inputs(
                            ctx,
                            &inputs,
                            attribute_prototype_id,
                            schema_variant_id,
                        )
                        .await?;
                    }
                }

                for existing_arg in existing_args.drain(..) {
                    if !inputs.iter().any(
                        |&LeafInput {
                             func_argument_id, ..
                         }| func_argument_id == existing_arg.id,
                    ) {
                        FuncArgument::remove(ctx, existing_arg.id).await?;
                    }
                }

                func_id
            }
            LeafBindingPrototype::Overlay(leaf_prototype_id) => {
                LeafPrototype::update_inputs(ctx, leaf_prototype_id, input_locations)
                    .await
                    .map_err(Box::new)?;
                LeafPrototype::func_id(ctx, leaf_prototype_id)
                    .await
                    .map_err(Box::new)?
            }
        };

        FuncBinding::for_func_id(ctx, func_id).await
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.leaf.delete_leaf_overlay_func_binding"
    )]
    pub async fn delete_leaf_overlay_func_binding(
        ctx: &DalContext,
        leaf_prototype_id: LeafPrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        let schemas = LeafPrototype::schemas(ctx, leaf_prototype_id)
            .await
            .map_err(Box::new)?;
        let eventual_parent = EventualParent::Schemas(schemas);
        LeafPrototype::remove(ctx, leaf_prototype_id)
            .await
            .map_err(Box::new)?;

        Ok(eventual_parent)
    }

    /// Deletes the attribute prototype for the given [`LeafKind`], including
    /// deleting the existing prototype arguments and the created attribute
    /// value/prop beneath the Root Prop node for the [`LeafKind`].
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.leaf.delete_leaf_func_binding"
    )]
    pub async fn delete_leaf_func_binding(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        // don't delete binding if parent is locked
        let eventual_parent =
            AttributeBinding::find_eventual_parent(ctx, attribute_prototype_id).await?;
        eventual_parent.error_if_locked(ctx).await?;

        // Cache the prop ID before we delete the prototype.
        let prop_id = AttributePrototype::prop_id(ctx, attribute_prototype_id).await?;

        // Delete all attribute prototype arguments for the given prototype.
        for attribute_prototype_argument_id in
            AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?
        {
            AttributePrototypeArgument::remove(ctx, attribute_prototype_argument_id).await?;
        }

        // Delete all attribute values using the prototype (all components who did not override the leaf function) and
        // then delete the prototype itself.
        for attribute_value_id in
            AttributePrototype::attribute_value_ids(ctx, attribute_prototype_id).await?
        {
            AttributeValue::remove(ctx, attribute_value_id).await?;
        }
        AttributePrototype::remove(ctx, attribute_prototype_id).await?;

        // Every prop must have at least one prototype, so if we deleted the last one, we must add
        // back the default.
        if let Some(prop_id) = prop_id {
            let remaining_prototypes = Prop::prototypes_by_key(ctx, prop_id).await?;

            if remaining_prototypes.is_empty() {
                let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset)
                    .await
                    .map_err(Box::new)?;
                let attribute_prototype = AttributePrototype::new(ctx, func_id)
                    .await
                    .map_err(Box::new)?;
                Prop::add_edge_to_attribute_prototype(
                    ctx,
                    prop_id,
                    attribute_prototype.id(),
                    EdgeWeightKind::Prototype(None),
                )
                .await?;
            }
        } else {
            warn!(si.error.message = "no prop found for attribute prototype when deleting leaf func binding", %attribute_prototype_id);
        }

        Ok(eventual_parent)
    }

    pub(crate) async fn compile_leaf_func_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<String> {
        let attribute_prototypes = AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
        let mut schema_variant_ids = vec![];
        for attribute_prototype_id in attribute_prototypes {
            match AttributeBinding::find_eventual_parent(ctx, attribute_prototype_id).await? {
                EventualParent::SchemaVariant(schema_variant_id) => {
                    schema_variant_ids.push(schema_variant_id)
                }
                EventualParent::Component(component_id) => {
                    // we probably want to grab the attribute value tree, but we'll defer to
                    // the prop tree for now
                    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
                    schema_variant_ids.push(schema_variant_id);
                }
                EventualParent::Schemas(schemas) => {
                    for schema_id in schemas {
                        for variant_id in Schema::list_schema_variant_ids(ctx, schema_id).await? {
                            schema_variant_ids.push(variant_id);
                        }
                    }
                }
            }
        }
        let mut ts_type = "type Input = {\n".to_string();
        let inputs = Self::list_leaf_function_inputs(ctx, func_id).await?;
        for input_location in inputs {
            let input_property = format!(
                "{}?: {} | null;\n",
                input_location.arg_name(),
                Self::get_per_variant_types_for_prop_path(
                    ctx,
                    &schema_variant_ids,
                    &input_location.prop_path(),
                )
                .await?
            );
            ts_type.push_str(&input_property);
        }
        ts_type.push_str("};");

        Ok(ts_type)
    }

    async fn get_per_variant_types_for_prop_path(
        ctx: &DalContext,
        variant_ids: &[SchemaVariantId],
        path: &PropPath,
    ) -> FuncBindingResult<String> {
        let mut per_variant_types = vec![];

        for variant_id in variant_ids {
            let prop_id = Prop::find_prop_id_by_path(ctx, *variant_id, path).await?;
            let ts_type = Prop::ts_type(ctx, prop_id).await?;

            if !per_variant_types.contains(&ts_type) {
                per_variant_types.push(ts_type);
            }
        }

        Ok(per_variant_types.join(" | "))
    }
}
