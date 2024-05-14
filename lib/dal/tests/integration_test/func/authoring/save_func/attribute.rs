use dal::attribute::prototype::argument::value_source::ValueSource;
use dal::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use dal::func::argument::{FuncArgument, FuncArgumentId, FuncArgumentKind};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::func::{
    AttributePrototypeArgumentBag, AttributePrototypeBag, FuncArgumentBag, FuncAssociations,
};
use dal::prop::PropPath;
use dal::{
    AttributePrototype, AttributePrototypeId, DalContext, Func, FuncId, Prop, Schema, SchemaVariant,
};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use std::collections::HashSet;

use crate::integration_test::func::authoring::save_func::save_func_setup;

#[test]
async fn modify_add_delete(ctx: &mut DalContext) {
    let (func_id, saved_func) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;
    base_assertions_for_attribute_funcs(ctx, func_id, &saved_func).await;

    // Scenario 1: modify the type of the existing func argument.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, mut arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());
        let cached_func_argument_id = argument.id;

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![FuncArgumentBag {
                    id: cached_func_argument_id,
                    name: argument.name,
                    kind: FuncArgumentKind::Boolean,
                    element_kind: None,
                }],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that it updated.
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
        let (_, mut arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            argument.kind              // actual
        );
        assert_eq!(
            cached_func_argument_id, // expected
            argument.id              // actual
        );

        // Check the actual func argument and that exactly one still exists.
        let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
            .await
            .expect("could not list ids for func");
        let func_argument_id = func_argument_ids.pop().expect("empty arguments");
        assert!(func_argument_ids.is_empty());
        assert_eq!(
            cached_func_argument_id, // expected
            func_argument_id         // actual
        );
        let func_argument = FuncArgument::get_by_id_or_error(ctx, func_argument_id)
            .await
            .expect("could not get func argument");
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            func_argument.kind         // actual
        );
    }

    // Scenario 2: add a func argument.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, mut arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        arguments.push(FuncArgumentBag {
            id: FuncArgumentId::NONE,
            name: "armillary".to_string(),
            kind: FuncArgumentKind::String,
            element_kind: None,
        });

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments,
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that we have the two arguments.
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
        let (mut prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert_eq!(1, prototype.prototype_arguments.len());
        assert_eq!(2, arguments.len());
    }

    // Scenario 3: delete all func arguments and update some metadata.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, _arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            Some("updated-display-name".to_string()),
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: Vec::new(),
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure the updated func looks as we expect.
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

        let (mut prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(arguments.is_empty());
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert!(prototype.prototype_arguments.is_empty());
    }
}

#[test]
async fn delete_add_modify(ctx: &mut DalContext) {
    let (func_id, saved_func) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;
    base_assertions_for_attribute_funcs(ctx, func_id, &saved_func).await;

    // Scenario 1: delete all func arguments.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, _arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            Some("updated-display-name".to_string()),
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: Vec::new(),
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure the updated func looks as we expect.
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

        let (mut prototypes, arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(arguments.is_empty());
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert!(prototype.prototype_arguments.is_empty());
    }

    // Scenario 2: add a func argument.
    let new_func_argument_id = {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        assert!(arguments.is_empty());

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![FuncArgumentBag {
                    id: FuncArgumentId::NONE,
                    name: "unity".to_string(),
                    kind: FuncArgumentKind::String,
                    element_kind: None,
                }],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that we have the one argument.
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
        let (mut prototypes, mut arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());

        // Ensure that there are no prototype arguments and return the func argument id.
        assert!(prototype.prototype_arguments.is_empty());
        argument.id
    };

    // Scenario 3: modify the type of the existing func argument.
    {
        let func = Func::get_by_id_or_error(ctx, func_id)
            .await
            .expect("could not get func by id");
        let func_view = FuncView::assemble(ctx, &func)
            .await
            .expect("could not assemble func view");
        let (prototypes, mut arguments) = func_view
            .associations
            .clone()
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());

        // Ensure that the collected func has the same argument.
        let cached_func_argument_id = argument.id;
        assert_eq!(
            new_func_argument_id,    // expected
            cached_func_argument_id, // actual
        );

        // Save the func and commit.
        FuncAuthoringClient::save_func(
            ctx,
            func_view.id,
            func_view.display_name,
            func_view.name,
            func_view.description,
            func_view.code,
            Some(FuncAssociations::Attribute {
                prototypes,
                arguments: vec![FuncArgumentBag {
                    id: cached_func_argument_id,
                    name: argument.name,
                    kind: FuncArgumentKind::Boolean,
                    element_kind: None,
                }],
            }),
        )
        .await
        .expect("unable to save func");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Ensure that it updated.
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
        let (mut prototypes, mut arguments) = func_view
            .associations
            .expect("could not get associations")
            .get_attribute_internals()
            .expect("could not get internals");
        let argument = arguments.pop().expect("empty arguments");
        assert!(arguments.is_empty());
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            argument.kind              // actual
        );
        assert_eq!(
            cached_func_argument_id, // expected
            argument.id              // actual
        );

        // Ensure that there are no prototype arguments and return the func argument id.
        let prototype = prototypes.pop().expect("empty prototypes");
        assert!(prototypes.is_empty());
        assert!(prototype.prototype_arguments.is_empty());

        // Check the actual func argument and that exactly one still exists.
        let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
            .await
            .expect("could not list ids for func");
        let func_argument_id = func_argument_ids.pop().expect("empty arguments");
        assert!(func_argument_ids.is_empty());
        assert_eq!(
            cached_func_argument_id, // expected
            func_argument_id         // actual
        );
        let func_argument = FuncArgument::get_by_id_or_error(ctx, func_argument_id)
            .await
            .expect("could not get func argument");
        assert_eq!(
            FuncArgumentKind::Boolean, // expected
            func_argument.kind         // actual
        );
    }
}

#[test]
async fn create_attribute_prototype_with_argument(ctx: &mut DalContext) {
    let (func_id, saved_func) = save_func_setup(ctx, "test:falloutEntriesToGalaxies").await;
    base_assertions_for_attribute_funcs(ctx, func_id, &saved_func).await;

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
    let (mut prototypes, arguments) = func_view
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
    prototypes.push(AttributePrototypeBag {
        id: AttributePrototypeId::NONE,
        component_id: None,
        schema_variant_id: Some(schema_variant_id),
        prop_id: Some(output_location_prop_id),
        output_socket_id: None,
        prototype_arguments: vec![AttributePrototypeArgumentBag {
            func_argument_id: func_argument.id,
            id: AttributePrototypeArgumentId::NONE,
            prop_id: Some(input_location_prop_id),
            input_socket_id: None,
        }],
    });

    // Save the func and commit.
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Attribute {
            prototypes,
            arguments,
        }),
    )
    .await
    .expect("unable to save func");
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
    let (prototypes, arguments) = func_view
        .associations
        .expect("could not get associations")
        .get_attribute_internals()
        .expect("could not get internals");
    assert_eq!(
        2,                // expected
        prototypes.len()  // actual
    );
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
    assert_eq!(
        vec![FuncArgumentBag {
            id: func_argument.id,
            name: "entries".to_string(),
            kind: FuncArgumentKind::Array,
            element_kind: Some(FuncArgumentKind::Object),
        }], // expected
        arguments // actual
    );
}

// Ensure everything looks as we expect before starting tests.
async fn base_assertions_for_attribute_funcs(
    ctx: &DalContext,
    func_id: FuncId,
    func_view: &FuncView,
) {
    let (prototypes, arguments) = func_view
        .associations
        .to_owned()
        .expect("could not get associations")
        .get_attribute_internals()
        .expect("could not get internals");

    // Ensure the prototypes look as we expect.
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

    // Ensure the arguments look as we expect.
    let func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
        .await
        .expect("could not list ids for func");
    assert_eq!(
        func_argument_ids, // expected
        arguments
            .iter()
            .map(|v| v.id)
            .collect::<Vec<FuncArgumentId>>()  // actual
    );
}
