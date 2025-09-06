use std::collections::HashSet;

use dal::{
    AttributePrototype,
    AttributePrototypeId,
    DalContext,
    Func,
    InputSocket,
    OutputSocket,
    Prop,
    Schema,
    SchemaVariant,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        value_source::ValueSource,
    },
    func::{
        argument::{
            FuncArgument,
            FuncArgumentKind,
        },
        binding::{
            AttributeArgumentBinding,
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            EventualParent,
            FuncBinding,
            attribute::AttributeBinding,
        },
        intrinsics::IntrinsicFunc,
    },
    prop::PropPath,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    helpers::ChangeSetTestHelpers,
    test,
};
pub use si_frontend_types;

use crate::integration_test::func::authoring::save_func::save_func_setup;

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
    let schema = Schema::get_by_name(ctx, "starfield")
        .await
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
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_summary = func
        .into_frontend_type(ctx)
        .await
        .expect("could not get func summary");

    assert_eq!(
        func_id,              // expected
        func_summary.func_id  // actual
    );
    let bindings = func_summary.bindings;
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
            AttributePrototypeArgument::value_source(ctx, attribute_prototype_argument_id)
                .await
                .expect("could not get value source");

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

// TODO restore this using subscriptions!
// #[test]
// async fn create_intrinsic_binding_then_unset(ctx: &mut DalContext) {
//     // Let's create a new asset
//     let asset_name = "paulsTestAsset".to_string();
//     let description = None;
//     let link = None;
//     let category = "Integration Tests".to_string();
//     let color = "#00b0b0".to_string();
//     let first_variant = VariantAuthoringClient::create_schema_and_variant(
//         ctx,
//         asset_name.clone(),
//         description.clone(),
//         link.clone(),
//         category.clone(),
//         color.clone(),
//     )
//     .await
//     .expect("Unable to create new asset");

//     let schema = first_variant
//         .schema(ctx)
//         .await
//         .expect("Unable to get the schema for the variant");

//     let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
//         .await
//         .expect("unable to get the default schema variant id");
//     assert_eq!(default_schema_variant, first_variant.id());

//     // Now let's update the asset and create two props, an input socket, and an output socket
//     let new_code = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build();\n const inputSocket = new SocketDefinitionBuilder()
//     .setName(\"one\")
//     .setArity(\"many\")
//     .build();\n const outputSocket = new SocketDefinitionBuilder()
//     .setName(\"output\")
//     .setArity(\"many\")
//     .build();\n const anotherProp = new PropBuilder().setName(\"anotherProp\").setKind(\"integer\").build();\n  return new AssetBuilder().addProp(myProp).addInputSocket(inputSocket).addOutputSocket(outputSocket).addProp(anotherProp).build()\n}".to_string();

//     VariantAuthoringClient::save_variant_content(
//         ctx,
//         first_variant.id(),
//         &schema.name,
//         first_variant.display_name(),
//         first_variant.category(),
//         first_variant.description(),
//         first_variant.link(),
//         first_variant
//             .get_color(ctx)
//             .await
//             .expect("get color from schema variant"),
//         first_variant.component_type(),
//         Some(new_code),
//     )
//     .await
//     .expect("save variant contents");

//     let updated_sv_id = VariantAuthoringClient::regenerate_variant(ctx, first_variant.id())
//         .await
//         .expect("regenerate asset");

//     let another_prop = Prop::find_prop_id_by_path(
//         ctx,
//         updated_sv_id,
//         &PropPath::new(["root", "domain", "anotherProp"]),
//     )
//     .await
//     .expect("able to find anotherProp prop");
//     let test_prop = Prop::find_prop_id_by_path(
//         ctx,
//         updated_sv_id,
//         &PropPath::new(["root", "domain", "testProp"]),
//     )
//     .await
//     .expect("able to find anotherProp prop");
//     let input_socket = InputSocket::find_with_name_or_error(ctx, "one", updated_sv_id)
//         .await
//         .expect("couldn't find input socket");
//     let output_socket = OutputSocket::find_with_name_or_error(ctx, "output", updated_sv_id)
//         .await
//         .expect("could not find output socket");

//     // let's set another prop to identity of test prop
//     let output_location = AttributeFuncDestination::Prop(another_prop);
//     let input_location = AttributeFuncArgumentSource::Prop(test_prop);
//     // find the func for si:identity
//     let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
//         .await
//         .expect("could not find identity func");
//     let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
//         .await
//         .expect("could not get args");
//     assert_eq!(intrinsic_arg.len(), 1);
//     let func_arg_id = intrinsic_arg.first().expect("is some");
//     let arguments: Vec<AttributeArgumentBinding> = vec![
//         (AttributeArgumentBinding {
//             func_argument_id: func_arg_id.id,
//             attribute_func_input_location: input_location,
//             attribute_prototype_argument_id: None,
//         }),
//     ];

//     AttributeBinding::upsert_attribute_binding(
//         ctx,
//         identity_func_id,
//         Some(EventualParent::SchemaVariant(updated_sv_id)),
//         output_location,
//         arguments,
//     )
//     .await
//     .expect("could not upsert identity func");

//     // let's set test prop to identity of input socket
//     let output_location = AttributeFuncDestination::Prop(test_prop);
//     let input_location = AttributeFuncArgumentSource::InputSocket(input_socket.id());
//     // find the func for si:identity
//     let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
//         .await
//         .expect("could not find identity func");
//     let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
//         .await
//         .expect("could not get args");
//     assert_eq!(intrinsic_arg.len(), 1);
//     let func_arg_id = intrinsic_arg.first().expect("is some");
//     let arguments: Vec<AttributeArgumentBinding> = vec![
//         (AttributeArgumentBinding {
//             func_argument_id: func_arg_id.id,
//             attribute_func_input_location: input_location,
//             attribute_prototype_argument_id: None,
//         }),
//     ];

//     AttributeBinding::upsert_attribute_binding(
//         ctx,
//         identity_func_id,
//         Some(EventualParent::SchemaVariant(updated_sv_id)),
//         output_location,
//         arguments,
//     )
//     .await
//     .expect("could not upsert identity func");

//     // let's set output socket to identity of another prop
//     let output_location = AttributeFuncDestination::OutputSocket(output_socket.id());
//     let input_location = AttributeFuncArgumentSource::Prop(another_prop);
//     // find the func for si:identity
//     let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
//         .await
//         .expect("could not find identity func");
//     let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
//         .await
//         .expect("could not get args");
//     assert_eq!(intrinsic_arg.len(), 1);
//     let func_arg_id = intrinsic_arg.first().expect("is some");
//     let arguments: Vec<AttributeArgumentBinding> = vec![
//         (AttributeArgumentBinding {
//             func_argument_id: func_arg_id.id,
//             attribute_func_input_location: input_location,
//             attribute_prototype_argument_id: None,
//         }),
//     ];

//     AttributeBinding::upsert_attribute_binding(
//         ctx,
//         identity_func_id,
//         Some(EventualParent::SchemaVariant(updated_sv_id)),
//         output_location,
//         arguments,
//     )
//     .await
//     .expect("could not upsert identity func");

//     ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
//         .await
//         .expect("could not update");

//     // Let's ensure that our latest props/sockets are visible in the component
//     let component = create_component_for_default_schema_name_in_default_view(
//         ctx,
//         schema.name.clone(),
//         "demo component 2",
//     )
//     .await
//     .expect("could not create component");
//     let connected_component =
//         create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "lego")
//             .await
//             .expect("could not create component");
//     // connect the two components
//     connect_components_with_socket_names(
//         ctx,
//         connected_component.id(),
//         "one",
//         component.id(),
//         "one",
//     )
//     .await
//     .expect("could not create connection");

//     ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
//         .await
//         .expect("could not update");

//     // let's make sure we can see the identity func for this schema variant
//     let maybe_funcs = SchemaVariant::all_funcs(ctx, updated_sv_id)
//         .await
//         .expect("could not get all funcs for sv");
//     let intrinsic = maybe_funcs
//         .into_iter()
//         .find(|func| func.is_intrinsic() && (func.backend_kind == FuncBackendKind::Identity))
//         .expect("is some");
//     let attributes = FuncBinding::get_attribute_bindings_for_func_id(ctx, intrinsic.id)
//         .await
//         .expect("could not get bindings");
//     let prototypes = attributes
//         .into_iter()
//         .filter(|p| p.eventual_parent == EventualParent::SchemaVariant(updated_sv_id))
//         .collect_vec();
//     assert_eq!(prototypes.len(), 3);
//     assert!(prototypes.into_iter().any(|binding| {
//         binding.output_location == AttributeFuncDestination::Prop(another_prop)
//             || binding.output_location == AttributeFuncDestination::Prop(test_prop)
//             || binding.output_location == AttributeFuncDestination::OutputSocket(output_socket.id())
//     }));

//     // Change attribute value for one on the component
//     update_attribute_value_for_component(
//         ctx,
//         connected_component.id(),
//         &["root", "domain", "one"],
//         serde_json::Value::String("test".to_string()),
//     )
//     .await
//     .expect("could not update attribute value");

//     ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
//         .await
//         .expect("could not update");
//     // check the other prop to make sure the value propagated

//     let test_prop_path = &["root", "domain", "testProp"];
//     let another_test_prop_path = &["root", "domain", "anotherProp"];

//     // check test prop to make sure the value propagated
//     let value = get_attribute_value_for_component(ctx, component.id(), test_prop_path)
//         .await
//         .expect("could not get attribute value");
//     assert_eq!(serde_json::Value::String("test".to_string()), value);

//     // check another prop too
//     let value = get_attribute_value_for_component(ctx, component.id(), another_test_prop_path)
//         .await
//         .expect("could not get attribute value");
//     assert_eq!(serde_json::Value::String("test".to_string()), value);

//     // check output socket
//     let value = get_component_output_socket_value(ctx, component.id(), "output")
//         .await
//         .expect("could not get attribute value")
//         .expect("value is empty");

//     assert_eq!(serde_json::Value::String("test".to_string()), value);

//     // let's set another_prop to unset now - which should clear the value fot that component
//     // and the output socket
//     let output_location = AttributeFuncDestination::Prop(another_prop);

//     let unset_func = Func::find_intrinsic(ctx, IntrinsicFunc::Unset)
//         .await
//         .expect("could not find unset func");
//     AttributeBinding::upsert_attribute_binding(
//         ctx,
//         unset_func,
//         Some(EventualParent::SchemaVariant(updated_sv_id)),
//         output_location,
//         vec![],
//     )
//     .await
//     .expect("could not upsert unset func");

//     ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
//         .await
//         .expect("could not commit");
//     // let's make sure we can see the identity func for this schema variant
//     let maybe_funcs = SchemaVariant::all_funcs(ctx, updated_sv_id)
//         .await
//         .expect("could not get all funcs for sv");
//     let intrinsic = maybe_funcs
//         .into_iter()
//         .find(|func| func.is_intrinsic() && (func.backend_kind == FuncBackendKind::Unset))
//         .expect("is some");
//     let attributes = FuncBinding::get_attribute_bindings_for_func_id(ctx, intrinsic.id)
//         .await
//         .expect("could not get bindings");
//     let prototypes = attributes
//         .into_iter()
//         .filter(|p| p.eventual_parent == EventualParent::SchemaVariant(updated_sv_id))
//         .collect_vec();
//     assert!(
//         prototypes
//             .into_iter()
//             .any(|binding| binding.output_location == AttributeFuncDestination::Prop(another_prop))
//     );

//     // let's make sure another_prop was cleared!
//     let value = get_attribute_value_for_component_opt(ctx, component.id(), another_test_prop_path)
//         .await
//         .expect("could not get attribute value");

//     assert!(value.is_none());

//     // check the output socket too
//     let value = get_component_output_socket_value(ctx, component.id(), "output")
//         .await
//         .expect("could not get attribute value");
//     assert!(value.is_none());
// }

#[test]
async fn invalid_identity_bindings(ctx: &mut DalContext) {
    // Let's create a new asset
    let asset_name = "paulsTestAsset".to_string();
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let first_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await
    .expect("Unable to create new asset");

    let schema = first_variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get the default schema variant id");
    assert_eq!(default_schema_variant, first_variant.id());

    // Now let's update the asset and create two props, an input socket, and an output socket
    let new_code = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build();\n const inputSocket = new SocketDefinitionBuilder()
        .setName(\"input\")
        .setArity(\"many\")
        .build();\n const outputSocket = new SocketDefinitionBuilder()
        .setName(\"output\")
        .setArity(\"many\")
        .build();\n const anotherProp = new PropBuilder().setName(\"anotherProp\").setKind(\"integer\").build();\n  return new AssetBuilder().addProp(myProp).addInputSocket(inputSocket).addOutputSocket(outputSocket).addProp(anotherProp).build()\n}".to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        first_variant.id(),
        &schema.name,
        first_variant.display_name(),
        first_variant.category(),
        first_variant.description(),
        first_variant.link(),
        first_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        first_variant.component_type(),
        Some(new_code),
    )
    .await
    .expect("save variant contents");

    let updated_sv_id = VariantAuthoringClient::regenerate_variant(ctx, first_variant.id())
        .await
        .expect("regenerate asset");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not update");

    let another_prop = Prop::find_prop_id_by_path(
        ctx,
        updated_sv_id,
        &PropPath::new(["root", "domain", "anotherProp"]),
    )
    .await
    .expect("able to find anotherProp prop");
    let test_prop = Prop::find_prop_id_by_path(
        ctx,
        updated_sv_id,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await
    .expect("able to find anotherProp prop");
    let input_socket = InputSocket::find_with_name_or_error(ctx, "input", updated_sv_id)
        .await
        .expect("couldn't find input socket");
    let output_socket = OutputSocket::find_with_name_or_error(ctx, "output", updated_sv_id)
        .await
        .expect("could not find output socket");

    // ensure we can't create a cycle when checking
    // have test prop take another prop, and another prop take test prop
    let cycle_check_guard = ctx
        .workspace_snapshot()
        .expect("could not get snapshot")
        .enable_cycle_check()
        .await;
    // test prop can't take input from itself
    let output_location = AttributeFuncDestination::Prop(test_prop);
    let input_location = AttributeFuncArgumentSource::Prop(test_prop);
    // find the func for si:identity
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
        .await
        .expect("could not get args");
    assert_eq!(intrinsic_arg.len(), 1);
    let func_arg_id = intrinsic_arg.first().expect("is some");
    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: func_arg_id.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    let attempt = AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        Some(EventualParent::SchemaVariant(updated_sv_id)),
        output_location,
        arguments,
    )
    .await;
    assert!(attempt.is_err());
    // output socket can't take input from itself
    let output_location = AttributeFuncDestination::OutputSocket(output_socket.id());
    let input_location = AttributeFuncArgumentSource::OutputSocket(output_socket.id());
    // find the func for si:identity
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
        .await
        .expect("could not get args");
    assert_eq!(intrinsic_arg.len(), 1);
    let func_arg_id = intrinsic_arg.first().expect("is some");
    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: func_arg_id.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    let attempt = AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        Some(EventualParent::SchemaVariant(updated_sv_id)),
        output_location,
        arguments,
    )
    .await;
    assert!(attempt.is_err());

    // input socket can't take inputs from anything
    let output_location = AttributeFuncDestination::InputSocket(input_socket.id());
    let input_location = AttributeFuncArgumentSource::OutputSocket(output_socket.id());
    // find the func for si:identity
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
        .await
        .expect("could not get args");
    assert_eq!(intrinsic_arg.len(), 1);
    let func_arg_id = intrinsic_arg.first().expect("is some");
    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: func_arg_id.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    let attempt = AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        Some(EventualParent::SchemaVariant(updated_sv_id)),
        output_location,
        arguments,
    )
    .await;
    assert!(attempt.is_err());

    // prop can't take inputs from output socket
    let output_location = AttributeFuncDestination::Prop(test_prop);
    let input_location = AttributeFuncArgumentSource::OutputSocket(output_socket.id());
    // find the func for si:identity
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
        .await
        .expect("could not get args");
    assert_eq!(intrinsic_arg.len(), 1);
    let func_arg_id = intrinsic_arg.first().expect("is some");
    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: func_arg_id.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    let attempt = AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        Some(EventualParent::SchemaVariant(updated_sv_id)),
        output_location,
        arguments,
    )
    .await;
    assert!(attempt.is_err());

    let output_location = AttributeFuncDestination::Prop(test_prop);
    let input_location = AttributeFuncArgumentSource::Prop(another_prop);
    // find the func for si:identity
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
        .await
        .expect("could not get args");
    assert_eq!(intrinsic_arg.len(), 1);
    let func_arg_id = intrinsic_arg.first().expect("is some");
    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: func_arg_id.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    let _attempt = AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        Some(EventualParent::SchemaVariant(updated_sv_id)),
        output_location,
        arguments,
    )
    .await
    .expect("unable to upsert binding");

    let output_location = AttributeFuncDestination::Prop(another_prop);
    let input_location = AttributeFuncArgumentSource::Prop(test_prop);
    // find the func for si:identity
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let intrinsic_arg = FuncArgument::list_for_func(ctx, identity_func_id)
        .await
        .expect("could not get args");
    assert_eq!(intrinsic_arg.len(), 1);
    let func_arg_id = intrinsic_arg.first().expect("is some");
    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: func_arg_id.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    let attempt = AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        Some(EventualParent::SchemaVariant(updated_sv_id)),
        output_location,
        arguments,
    )
    .await;
    assert!(attempt.is_err());
    drop(cycle_check_guard);
}

#[test]
async fn detach_attribute_func(ctx: &mut DalContext) {
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot");
    let default_schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "starfield")
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
