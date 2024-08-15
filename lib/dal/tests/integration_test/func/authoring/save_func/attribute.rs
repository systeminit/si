use std::collections::HashSet;

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
use dal::{AttributePrototype, AttributePrototypeId, DalContext, Prop, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn create_attribute_prototype_with_attribute_prototype_argument(ctx: &mut DalContext) {
    let (func_id, _) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;

    // Ensure the bindings look as we expect.
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
        .expect("could not find schema")
        .expect("schema not found");
    let schema_variant_id = SchemaVariant::get_unlocked_for_schema(ctx, schema.id())
        .await
        .expect("no schema variant found")
        .expect("has an unlocked variant")
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

    // Get the func view.
    assert_eq!(
        1,              // expected
        bindings.len()  // actual
    );
    let existing_attribute_prototype_id = bindings
        .first()
        .expect("empty attribute prototypes")
        .attribute_prototype_id;

    // Add a new prototype with a new prototype argument. Use the existing func argument.
    let func_argument = FuncArgument::find_by_name_for_func(ctx, "entries", func_id)
        .await
        .expect("could not perform find by name for func")
        .expect("func argument not found");

    // create the new attribute prototype and commit

    AttributeBinding::upsert_attribute_binding(
        ctx,
        func_id,
        Some(EventualParent::SchemaVariant(schema_variant_id)),
        AttributeFuncDestination::Prop(output_location_prop_id),
        vec![AttributeArgumentBinding {
            func_argument_id: func_argument.id,
            attribute_prototype_argument_id: None,
            attribute_func_input_location: AttributeFuncArgumentSource::Prop(
                input_location_prop_id,
            ),
        }],
    )
    .await
    .expect("could not upsert attribute binding");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that everything looks as we expect with the new prototype and prototype argument.

    let bindings = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get bindings");

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

    // Gather up the expected bags.
    let mut expected = HashSet::new();
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

            expected.insert(AttributeBinding {
                func_id,
                attribute_prototype_id,
                eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
                output_location: AttributeFuncDestination::Prop(existing_output_location_prop_id),
                argument_bindings: vec![AttributeArgumentBinding {
                    func_argument_id: func_argument.id,
                    attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
                    attribute_func_input_location: AttributeFuncArgumentSource::InputSocket(
                        input_socket_id,
                    ),
                }],
            });
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
            expected.insert(AttributeBinding {
                func_id,
                attribute_prototype_id,
                eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
                output_location: AttributeFuncDestination::Prop(output_location_prop_id),
                argument_bindings: vec![AttributeArgumentBinding {
                    func_argument_id: func_argument.id,
                    attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
                    attribute_func_input_location: AttributeFuncArgumentSource::Prop(prop_id),
                }],
            });
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
