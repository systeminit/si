use crate::{
    attribute::prototype::argument::AttributePrototypeArgument,
    func::argument::FuncArgument,
    prop::PropPath,
    schema::variant::leaves::{LeafInput, LeafInputLocation, LeafKind},
    AttributePrototype, AttributePrototypeId, Component, ComponentId, DalContext, Func, FuncId,
    Prop, SchemaVariant, SchemaVariantId,
};

use super::{
    attribute::{find_eventual_parent, EventualParent},
    EventualParent, FuncBinding, FuncBindingDiscriminants, FuncBindings, FuncBindingsError,
    FuncBindingsResult,
};

pub(crate) async fn assemble_code_gen_bindings(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<Vec<FuncBinding>> {
    let inputs = list_leaf_function_inputs(ctx, func_id).await?;
    let mut bindings = vec![];
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;

    for attribute_prototype_id in attribute_prototype_ids {
        let eventual_parent = find_eventual_parent(ctx, attribute_prototype_id).await?;

        bindings.push(FuncBinding::CodeGeneration {
            eventual_parent,
            func_id,
            inputs: inputs.clone(),
            attribute_prototype_id: Some(attribute_prototype_id),
        });
    }
    Ok(bindings)
}
pub(crate) async fn assemble_qualification_bindings(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<Vec<FuncBinding>> {
    let inputs = list_leaf_function_inputs(ctx, func_id).await?;
    let mut bindings = vec![];
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;

    for attribute_prototype_id in attribute_prototype_ids {
        let eventual_parent = find_eventual_parent(ctx, attribute_prototype_id).await?;
        bindings.push(FuncBinding::Qualification {
            eventual_parent,
            func_id,
            inputs: inputs.clone(),
            attribute_prototype_id: Some(attribute_prototype_id),
        });
    }
    Ok(bindings)
}
async fn list_leaf_function_inputs(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<Vec<LeafInputLocation>> {
    Ok(FuncArgument::list_for_func(ctx, func_id)
        .await?
        .iter()
        .filter_map(|arg| LeafInputLocation::maybe_from_arg_name(&arg.name))
        .collect())
}

pub async fn create_leaf_func_binding(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: Option<SchemaVariantId>,
    component_id: Option<ComponentId>,
    leaf_kind: LeafKind,
    inputs: &[LeafInputLocation],
) -> FuncBindingsResult<FuncBindings> {
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    if let Some(schema_variant_id) = schema_variant_id {
        SchemaVariant::upsert_leaf_function(ctx, schema_variant_id, leaf_kind, inputs, &func)
            .await?;
    }

    if let Some(_component_id) = component_id {
        //brit todo create this func
        // let attribute_prototype_id =
        //     Component::upsert_leaf_function(ctx, component_id, leaf_kind, inputs, &func).await?;
    }
    let new_bindings = FuncBindings::from_func_id(ctx, func_id).await?;
    Ok(new_bindings)
}

pub async fn update_leaf_func_binding_inner(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
    schema_variant_id: Option<SchemaVariantId>,
    component_id: Option<ComponentId>,
    input_locations: &[LeafInputLocation],
) -> FuncBindingsResult<FuncBindings> {
    // find the prototype
    let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
    // update the input locations
    let mut existing_args = FuncArgument::list_for_func(ctx, func_id).await?;
    let mut inputs = vec![];
    for location in input_locations {
        let arg_name = location.arg_name();
        let arg = match existing_args
            .iter()
            .find(|arg| arg.name.as_str() == arg_name)
        {
            Some(existing_arg) => existing_arg.clone(),
            None => FuncArgument::new(ctx, arg_name, location.arg_kind(), None, func_id).await?,
        };

        inputs.push(LeafInput {
            location: *location,
            func_argument_id: arg.id,
        });
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
    match find_eventual_parent(ctx, attribute_prototype_id).await? {
        EventualParent::SchemaVariant(schema_variant_id) => {
            SchemaVariant::upsert_leaf_function_inputs(
                ctx,
                inputs.as_slice(),
                attribute_prototype_id,
                schema_variant_id,
            )
            .await?;
        }
        EventualParent::Component(component_id) => {
            // brit todo : write this func
            // Component::upsert_leaf_function_inputs(
            //     ctx,
            //     inputs.as_slice(),
            //     attribute_prototype_id,
            //     component_id,
            // )
            // .await?;
        }
    }

    let updated_bindings = FuncBindings::from_func_id(ctx, func_id).await?;

    Ok(updated_bindings)
}

pub async fn delete_leaf_func_binding(
    ctx: &DalContext,
    attribute_prototype_id: AttributePrototypeId,
) -> FuncBindingsResult<FuncBindings> {
    let func_id = AttributePrototype::func_id(ctx, attribute_prototype_id).await?;
    let current_attribute_prototype_arguments =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;
    for apa in current_attribute_prototype_arguments {
        AttributePrototypeArgument::remove(ctx, apa).await?;
    }
    AttributePrototype::remove(ctx, attribute_prototype_id).await?;
    let updated_bindings = FuncBindings::from_func_id(ctx, func_id).await?;

    Ok(updated_bindings)

    //brit todo delete the attribute value for the thing too
}

pub(crate) async fn compile_leaf_func_types(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncBindingsResult<String> {
    let attribute_prorotypes = AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
    let mut schema_variant_ids = vec![];
    for attribute_prototype_id in attribute_prorotypes {
        match find_eventual_parent(ctx, attribute_prototype_id).await? {
            EventualParent::SchemaVariant(schema_variant_id) => {
                schema_variant_ids.push(schema_variant_id)
            }
            EventualParent::Component(component_id) => {
                // we probably want to grab the attribute value tree, but we'll defer to
                // the prop tree for now
                let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
                schema_variant_ids.push(schema_variant_id);
            }
        }
    }
    let mut ts_type = "type Input = {\n".to_string();
    let inputs = list_leaf_function_inputs(ctx, func_id).await?;
    for input_location in inputs {
        let input_property = format!(
            "{}?: {} | null;\n",
            input_location.arg_name(),
            get_per_variant_types_for_prop_path(
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
) -> FuncBindingsResult<String> {
    let mut per_variant_types = vec![];

    for variant_id in variant_ids {
        let prop = Prop::find_prop_by_path(ctx, *variant_id, path).await?;
        let ts_type = prop.ts_type(ctx).await?;

        if !per_variant_types.contains(&ts_type) {
            per_variant_types.push(ts_type);
        }
    }

    Ok(per_variant_types.join(" | "))
}
