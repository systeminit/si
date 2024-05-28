use std::collections::HashSet;

use crate::integration_test::func::authoring::save_func::save_func_setup;
use dal::attribute::prototype::argument::value_source::ValueSource;
use dal::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::func::{AttributePrototypeArgumentBag, AttributePrototypeBag};
use dal::prop::PropPath;
use dal::{
    AttributePrototype, AttributePrototypeId, DalContext, Func, Prop, Schema, SchemaVariant,
};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn create_attribute_prototype_with_attribute_prototype_argument(ctx: &mut DalContext) {
    let (func_id, saved_func) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;

    // Ensure the prototypes look as we expect.
    let prototypes = saved_func
        .associations
        .to_owned()
        .expect("could not get associations")
        .get_attribute_internals()
        .expect("could not get internals");
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list ids for func id");
    assert_eq!(
        attribute_prototype_ids, // expected
        prototypes
            .iter()
            .map(|v| v.id)
            .collect::<Vec<AttributePrototypeId>>()  // actual
    );

    // Cache the variables we need.
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema")
        .expect("schema not found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("no schema variant found");
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
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    let prototypes = func_view
        .associations
        .clone()
        .expect("could not get associations")
        .get_attribute_internals()
        .expect("could not get internals");
    assert_eq!(
        1,                // expected
        prototypes.len()  // actual
    );
    let existing_attribute_prototype_id =
        prototypes.first().expect("empty attribute prototypes").id;

    // Add a new prototype with a new prototype argument. Use the existing func argument.
    let func_argument = FuncArgument::find_by_name_for_func(ctx, "entries", func_id)
        .await
        .expect("could not perform find by name for func")
        .expect("func argument not found");
    let prototype_arguments = vec![AttributePrototypeArgumentBag {
        func_argument_id: func_argument.id,
        id: AttributePrototypeArgumentId::NONE,
        prop_id: Some(input_location_prop_id),
        input_socket_id: None,
    }];

    // create the new attribute prototype and commit
    FuncAuthoringClient::create_attribute_prototype(
        ctx,
        func_view.id,
        schema_variant_id,
        None,
        Some(output_location_prop_id),
        None,
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
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    assert_eq!(
        func_id,      // expected
        func_view.id  // actual
    );
    let prototypes = func_view
        .associations
        .expect("could not get associations")
        .get_attribute_internals()
        .expect("could not get internals");
    assert_eq!(
        2,                // expected
        prototypes.len()  // actual
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

            expected.insert(AttributePrototypeBag {
                id: attribute_prototype_id,
                component_id: None,
                schema_variant_id: Some(schema_variant_id),
                prop_id: Some(existing_output_location_prop_id),
                output_socket_id: None,
                prototype_arguments: vec![AttributePrototypeArgumentBag {
                    func_argument_id: func_argument.id,
                    id: attribute_prototype_argument_id,
                    prop_id: None,
                    input_socket_id: Some(input_socket_id),
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
            expected.insert(AttributePrototypeBag {
                id: attribute_prototype_id,
                component_id: None,
                schema_variant_id: Some(schema_variant_id),
                prop_id: Some(output_location_prop_id),
                output_socket_id: None,
                prototype_arguments: vec![AttributePrototypeArgumentBag {
                    func_argument_id: func_argument.id,
                    id: attribute_prototype_argument_id,
                    prop_id: Some(prop_id),
                    input_socket_id: None,
                }],
            });
        }
    }

    // Now that we have the expected prototypes, we can perform the final assertions.
    assert_eq!(
        expected,                                   // expected
        HashSet::from_iter(prototypes.into_iter()), // actual
    );
    let argument = arguments.pop().expect("empty func arguments");
    assert_eq!(func_argument.id, argument.id);
    assert_eq!("entries", argument.name.as_str());
    assert_eq!(FuncArgumentKind::Array, argument.kind);
    assert_eq!(Some(FuncArgumentKind::Object), argument.element_kind);
}
