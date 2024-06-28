use std::collections::HashMap;

use crate::{
    attribute::prototype::{
        argument::AttributePrototypeArgument, AttributePrototypeEventualParent,
    },
    func::{
        argument::{FuncArgument, FuncArgumentError},
        FuncKind,
    },
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
    AttributePrototype, AttributePrototypeId, AttributeValue, ComponentId, DalContext,
    EdgeWeightKind, Func, FuncId, OutputSocket, OutputSocketId, Prop, PropId, SchemaVariantId,
    WorkspaceSnapshotError,
};

use super::{
    AttributeArgumentBinding, FuncBinding, FuncBindings, FuncBindingsError, FuncBindingsResult,
};

pub(crate) async fn find_eventual_parent(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
) -> FuncBindingsResult<EventualParent> {
    let eventual_parent = AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
    let parent = match eventual_parent {
        AttributePrototypeEventualParent::Component(component_id) => {
            EventualParent::Component(component_id)
        }
        AttributePrototypeEventualParent::SchemaVariantFromInputSocket(schema_variant_id, _) => {
            EventualParent::SchemaVariant(schema_variant_id)
        }

        AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(schema_variant_id, _) => {
            EventualParent::SchemaVariant(schema_variant_id)
        }
        AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, _) => {
            EventualParent::SchemaVariant(schema_variant_id)
        }
    };
    Ok(parent)
}

pub(crate) async fn assemble_attribute_bindings(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<Vec<FuncBinding>> {
    let mut bindings = vec![];
    for attribute_prototype_id in AttributePrototype::list_ids_for_func_id(ctx, func_id).await? {
        let eventual_parent =
            AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
        let (component_id, schema_variant_id, prop_id, output_socket_id) = match eventual_parent {
            AttributePrototypeEventualParent::Component(component_id) => {
                (Some(component_id), None, None, None)
            }
            AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
                schema_variant_id,
                _,
            ) => (None, Some(schema_variant_id), None, None),
            AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
                schema_variant_id,
                output_socket_id,
            ) => (None, Some(schema_variant_id), None, Some(output_socket_id)),
            AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, prop_id) => {
                (None, Some(schema_variant_id), Some(prop_id), None)
            }
        };

        let attribute_prototype_argument_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;

        let mut argument_bindings = Vec::with_capacity(attribute_prototype_argument_ids.len());
        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            argument_bindings.push(
                AttributeArgumentBinding::assemble(ctx, attribute_prototype_argument_id).await?,
            );
        }
        bindings.push(FuncBinding::Attribute {
            func_id,
            attribute_prototype_id: Some(attribute_prototype_id),
            component_id,
            schema_variant_id,
            prop_id,
            output_socket_id,
            argument_bindings,
        });
    }
    Ok(bindings)
}

pub(crate) async fn upsert_attribute_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: Option<SchemaVariantId>,
    component_id: Option<ComponentId>,
    prop_id: Option<PropId>,
    output_socket_id: Option<OutputSocketId>,
    prototype_arguments: Vec<AttributeArgumentBinding>,
) -> FuncBindingsResult<FuncBindings> {
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    if func.kind != FuncKind::Attribute {
        return Err(FuncBindingsError::UnexpectedFuncKind(func.kind));
    }
    let attribute_prototype = AttributePrototype::new(ctx, func_id).await?;
    let attribute_prototype_id = attribute_prototype.id;
    let mut affected_attribute_value_ids = vec![];
    if let Some(prop_id) = prop_id {
        if let Some(_schema_variant_id) = schema_variant_id {
            // remove the existing attribute prototype and arguments
            if let Some(existing_proto) =
                AttributePrototype::find_for_prop(ctx, prop_id, &None).await?
            {
                delete_attribute_prototype_and_args(ctx, existing_proto).await?;
            }

            Prop::add_edge_to_attribute_prototype(
                ctx,
                prop_id,
                attribute_prototype.id,
                EdgeWeightKind::Prototype(None),
            )
            .await?;
        } else if let Some(component_id) = component_id {
            let attribute_value_ids = Prop::attribute_values_for_prop_id(ctx, prop_id).await?;

            for attribute_value_id in attribute_value_ids {
                if component_id == AttributeValue::component_id(ctx, attribute_value_id).await? {
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
    } else if let Some(output_socket_id) = output_socket_id {
        if let Some(_schema_variant_id) = schema_variant_id {
            // remove the existing attribute prototype and arguments
            if let Some(existing_proto) =
                AttributePrototype::find_for_output_socket(ctx, output_socket_id).await?
            {
                delete_attribute_prototype_and_args(ctx, existing_proto).await?;
            }
            OutputSocket::add_edge_to_attribute_prototype(
                ctx,
                output_socket_id,
                attribute_prototype.id,
                EdgeWeightKind::Prototype(None),
            )
            .await?;
        } else if let Some(component_id) = component_id {
            let attribute_value_ids =
                OutputSocket::attribute_values_for_output_socket_id(ctx, output_socket_id).await?;
            for attribute_value_id in attribute_value_ids {
                if component_id == AttributeValue::component_id(ctx, attribute_value_id).await? {
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
    } else {
        return Err(FuncBindingsError::NoOutputLocationGiven(func_id));
    }

    if !affected_attribute_value_ids.is_empty() {
        ctx.add_dependent_values_and_enqueue(affected_attribute_value_ids)
            .await?;
    }
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

        if let Some(input_socket_id) = arg.input_socket_id {
            attribute_prototype_argument
                .set_value_from_input_socket_id(ctx, input_socket_id)
                .await?;
        } else if let Some(prop_id) = arg.prop_id {
            attribute_prototype_argument
                .set_value_from_prop_id(ctx, prop_id)
                .await?;
        } else {
            return Err(FuncBindingsError::NoInputLocationGiven(
                attribute_prototype_id,
                arg.func_argument_id,
            ));
        }
    }
    let new_bindings = FuncBindings::from_func_id(ctx, func_id).await?;
    Ok(new_bindings)
}

pub(crate) async fn delete_attribute_prototype_and_args(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
) -> FuncBindingsResult<()> {
    let current_attribute_prototype_arguments =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;
    for apa in current_attribute_prototype_arguments {
        AttributePrototypeArgument::remove(ctx, apa).await?;
    }
    AttributePrototype::remove(ctx, attribute_prototype_id).await?;
    Ok(())
}
// this is what happens when you detach an attribute func
// async fn reset_attribute_binding(
//     ctx: &DalContext,
//     attribute_prototype_id: AttributePrototypeId,
// ) -> FuncBindingsResult<FuncBindings> {
//     let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
//     let schema_variant_json = get_schema_variant_json(ctx, schema_variant_id).await?;

//     AttributePrototype::reset_attribute_prototype(ctx, attribute_prototype_id, schema_variant_json)
//         .await?;

//     let new_binding = FuncBindings::from_func_id(ctx, func_id).await?;
//     Ok(new_binding)
// }

pub(crate) async fn compile_attribute_types(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<String> {
    let mut input_ts_types = "type Input = {\n".to_string();

    let mut output_ts_types = vec![];
    let mut argument_types = HashMap::new();
    let bindings = assemble_attribute_bindings(ctx, func_id).await?;
    for binding in bindings {
        if let FuncBinding::Attribute {
            func_id: _,
            attribute_prototype_id: _,
            component_id: _,
            schema_variant_id: _,
            prop_id,
            output_socket_id: _,
            argument_bindings,
        } = binding
        {
            for arg in argument_bindings {
                if let Some(prop_id) = arg.prop_id {
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
                let output_type = if let Some(output_prop_id) = prop_id {
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

// async fn get_schema_variant_json(
//     ctx: &DalContext,
//     schema_variant_id: SchemaVariantId,
// ) -> FuncBindingsResult<SchemaVariantJson> {
//     let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
//     let asset_func = schema_variant.asset_func_id();
//     if let Some(asset_func_id) = asset_func {
//         let asset_func = Func::get_by_id_or_error(ctx, asset_func_id).await?;
//         let json = VariantAuthoringClient::execute_asset_func(ctx, &asset_func).await?;
//         Ok(json)
//     }

// }
// pub async fn reset_attribute_prototype(
//     ctx: &DalContext,
//     attribute_prototype_id: AttributePrototypeId,
//     schema_variant_json: SchemaVariantJson,
// ) -> AttributePrototypeResult<()> {
//     // for a schema variant, we need to re-execute the schema definition, find the prop, and do the thing it says
//     // todo : this
//     let eventual_parent =
//         AttributePrototype::eventual_parent(ctx, attribute_prototype_id).await?;
//     match eventual_parent {
//         AttributePrototypeEventualParent::Component(_) => {
//             if let Some(attribute_value_id) =
//                 AttributePrototype::attribute_value_id(ctx, attribute_prototype_id).await?
//             {
//                 AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
//             }
//         }
//         AttributePrototypeEventualParent::SchemaVariantFromInputSocket(
//             schema_variant_id,
//             input_socket_id,
//         ) => {
//             // how did we even get here?
//             todo!()
//         }
//         AttributePrototypeEventualParent::SchemaVariantFromOutputSocket(
//             schema_variant_id,
//             output_socket_id,
//         ) => {
//             if !try_set_output_socket_value_from(ctx, output_socket_id, attribute_prototype_id)
//                 .await?
//             {
//                 Err("todo")
//             }
//         }
//         AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, prop_id) => {
//             todo!()
//         }
//     }
// }
// async fn try_set_output_socket_value_from(
//     ctx: &DalContext,
//     output_socket_id: OutputSocketId,
//     attribute_prototype_id: AttributePrototypeId,
// ) -> FuncBindingsResult<bool> {
//     let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
//     let schema_variant_json = get_schema_variant_json(ctx, schema_variant_id).await?;
//     let maybe_value_from = schema_variant_json
//         .output_sockets
//         .into_iter()
//         .filter_map(|output| {
//             if output.name == output_socket.name() {
//                 output.value_from
//             } else {
//                 None
//             }
//         })
//         .collect_vec();
//     if let Some(value_from) = maybe_value_from.pop() {
//         // set value from!
//         let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;
//         let current_attribute_prototype_arguments =
//             AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;
//         for apa in current_attribute_prototype_arguments {
//             AttributePrototypeArgument::remove(ctx, apa).await?;
//         }
//         AttributePrototype::update_func_by_id(ctx, attribute_prototype_id, identity_func_id)
//             .await?;
//         let func_arg_id = *FuncArgument::list_ids_for_func(ctx, identity_func_id)
//             .await?
//             .first()
//             .ok_or(FuncArgumentError::IntrinsicMissingFuncArgumentEdge(
//                 "identity".into(),
//                 identity_func_id,
//             ))?;
//         match value_from {
//             ValueFrom::InputSocket { socket_name } => {
//                 let input_socket =
//                     InputSocket::find_with_name_or_error(ctx, socket_name, schema_variant_id)
//                         .await?;
//                 AttributePrototypeArgument::new(ctx, attribute_prototype_id, func_arg_id)
//                     .await?
//                     .set_value_from_input_socket_id(ctx, input_socket.id())
//                     .await?
//             }
//             ValueFrom::OutputSocket { socket_name } => return Err("todo"),
//             ValueFrom::Prop { prop_path } => {
//                 let prop =
//                     Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(prop_path))
//                         .await?;
//                 AttributePrototypeArgument::new(ctx, attribute_prototype_id, func_arg_id)
//                     .await?
//                     .set_value_from_prop_id(ctx, prop)
//                     .await?;
//             }
//         }
//         Ok(true)
//     }
//     Ok(false)
// }
