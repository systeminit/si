use crate::integration_test::func::authoring::save_func::save_func_setup;
use dal::attribute::prototype::argument::value_source::ValueSource;
use dal::attribute::prototype::argument::AttributePrototypeArgument;
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::binding::attribute::AttributeBinding;
use dal::func::binding::{
    AttributeArgumentBinding, AttributeFuncArgumentSource, AttributeFuncDestination,
    EventualParent, FuncBinding,
};
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{
    AttributePrototype, AttributePrototypeId, DalContext, Func, Prop, Schema, SchemaVariant,
};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
pub use si_frontend_types;
use std::collections::HashSet;

#[test]
async fn create_attribute_prototype_with_attribute_prototype_argument(ctx: &mut DalContext) {
    let (func_id, _) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;

    // Ensure the prototypes look as we expect.
    let bindings = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get bindings");

    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list ids for func id");
    assert_eq!(
        attribute_prototype_ids, // expected
        bindings
            .iter()
            .map(|v| v.attribute_prototype_id)
            .collect::<Vec<AttributePrototypeId>>()  // actual
    );

    // Cache the variables we need.
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not execute find schema")
        .expect("schema not found");
    let schema_variant_id = SchemaVariant::get_unlocked_for_schema(ctx, schema.id())
        .await
        .expect("execute get_unlocked_for_schema")
        .expect("value is some")
        .id();

    let input_location_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "name"]),
    )
    .await
    .expect("could not find prop id by path");
    let output_location_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "possible_world_b", "wormhole_1"]),
    )
    .await
    .expect("could not find prop id by path");

    let attributes = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get attribute internals");

    assert_eq!(
        1,                // expected
        attributes.len()  // actual
    );
    let existing_attribute_prototype_id = attributes
        .first()
        .expect("empty attribute prototypes")
        .attribute_prototype_id;

    // Add a new prototype with a new prototype argument. Use the existing func argument.
    let func_argument = FuncArgument::find_by_name_for_func(ctx, "entries", func_id)
        .await
        .expect("could not perform find by name for func")
        .expect("func argument not found");

    FuncArgument::list_for_func(ctx, func_id)
        .await
        .expect("could list");
    let prototype_arguments = vec![AttributeArgumentBinding {
        func_argument_id: func_argument.id,
        attribute_prototype_argument_id: None,
        attribute_func_input_location: AttributeFuncArgumentSource::Prop(input_location_prop_id),
    }];

    // create the new attribute prototype and commit
    AttributeBinding::upsert_attribute_binding(
        ctx,
        func_id,
        Some(EventualParent::SchemaVariant(schema_variant_id)),
        AttributeFuncDestination::Prop(output_location_prop_id),
        prototype_arguments,
    )
    .await
    .expect("could not create attribute prototype");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that everything looks as we expect with the new prototype and prototype argument.
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_summary = func
        .into_frontend_type(ctx)
        .await
        .expect("could not get func summary");

    assert_eq!(
        func_id,                     // expected
        func_summary.func_id.into()  // actual
    );
    let bindings = func_summary.bindings.bindings;
    assert_eq!(
        2,              // expected
        bindings.len()  // actual
    );
    let mut arguments = FuncArgument::list_for_func(ctx, func_id)
        .await
        .expect("could not list func arguments");
    assert_eq!(
        1,               // expected
        arguments.len()  // actual
    );
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not find attribute prototype ids");
    assert_eq!(
        2,                             // expected
        attribute_prototype_ids.len()  // actual
    );

    // Gather the expected bindings.
    let mut expected: HashSet<si_frontend_types::FuncBinding> = HashSet::new();
    for attribute_prototype_id in attribute_prototype_ids {
        let mut attribute_prototype_argument_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
                .await
                .expect("could not list ids for prototype");
        let attribute_prototype_argument_id = attribute_prototype_argument_ids
            .pop()
            .expect("empty attribute prototype argument ids");
        let value_source =
            AttributePrototypeArgument::value_source_by_id(ctx, attribute_prototype_argument_id)
                .await
                .expect("could not get value source")
                .expect("value source not found");

        if existing_attribute_prototype_id == attribute_prototype_id {
            // Assemble the expected bag for the existing prototype.
            let input_socket_id = match value_source {
                ValueSource::InputSocket(input_socket_id) => input_socket_id,
                value_source => panic!("unexpected value source: {value_source:?}"),
            };
            let existing_output_location_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant_id,
                &PropPath::new(["root", "domain", "universe", "galaxies"]),
            )
            .await
            .expect("could not find prop id by path");
            expected.insert(
                FuncBinding::Attribute(AttributeBinding {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
                    output_location: AttributeFuncDestination::Prop(
                        existing_output_location_prop_id,
                    ),
                    argument_bindings: vec![AttributeArgumentBinding {
                        func_argument_id: func_argument.id,
                        attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
                        attribute_func_input_location: AttributeFuncArgumentSource::InputSocket(
                            input_socket_id,
                        ),
                    }],
                })
                .into(),
            );
        } else {
            // Assemble the expected bag for the new prototype.
            let prop_id = match value_source {
                ValueSource::Prop(prop_id) => prop_id,
                value_source => panic!("unexpected value source: {value_source:?}"),
            };
            assert_eq!(
                input_location_prop_id, // expected
                prop_id                 // actual
            );
            expected.insert(
                FuncBinding::Attribute(AttributeBinding {
                    func_id,
                    attribute_prototype_id,
                    eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
                    output_location: AttributeFuncDestination::Prop(output_location_prop_id),
                    argument_bindings: vec![AttributeArgumentBinding {
                        func_argument_id: func_argument.id,
                        attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
                        attribute_func_input_location: AttributeFuncArgumentSource::Prop(prop_id),
                    }],
                })
                .into(),
            );
        }
    }

    // Now that we have the expected prototypes, we can perform the final assertions.
    assert_eq!(
        expected,                                 // expected
        HashSet::from_iter(bindings.into_iter()), // actual
    );
    let argument = arguments.pop().expect("empty func arguments");
    assert_eq!(func_argument.id, argument.id);
    assert_eq!("entries", argument.name.as_str());
    assert_eq!(FuncArgumentKind::Array, argument.kind);
    assert_eq!(Some(FuncArgumentKind::Object), argument.element_kind);
}

#[test]
async fn detach_attribute_func(ctx: &mut DalContext) {
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot");
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let default_schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");

    // create unlocked copy
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, default_schema_variant_id)
            .await
            .expect("can create unlocked copy")
            .id();

    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    let total_funcs = funcs.len();

    // Detach one attribute func to the schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let attributes = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get bindings");
    let prototype = attributes
        .into_iter()
        .find(|p| p.eventual_parent == EventualParent::SchemaVariant(schema_variant_id))
        .expect("could not find schema variant");
    AttributeBinding::reset_attribute_binding(ctx, prototype.attribute_prototype_id)
        .await
        .expect("could not reset prototype");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see what's left.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    assert_eq!(
        total_funcs - 1, // expected
        funcs.len()      // actual
    );
    assert!(!funcs.iter().any(|summary| summary.id == func_id));
}
