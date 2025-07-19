use dal::{
    AttributePrototype,
    Component,
    DalContext,
    Func,
    InputSocket,
    OutputSocket,
    Prop,
    Schema,
    SchemaVariant,
    Secret,
    action::prototype::ActionKind,
    attribute::prototype::argument::AttributePrototypeArgument,
    func::{
        argument::{
            FuncArgument,
            FuncArgumentKind,
        },
        authoring::FuncAuthoringClient,
        binding::{
            AttributeArgumentBinding,
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            EventualParent,
            FuncBinding,
            action::ActionBinding,
            attribute::AttributeBinding,
            authentication::AuthBinding,
            leaf::LeafBinding,
        },
        intrinsics::IntrinsicFunc,
    },
    prop::PropPath,
    property_editor::values::PropertyEditorValues,
    schema::variant::{
        authoring::VariantAuthoringClient,
        leaves::{
            LeafInputLocation,
            LeafKind,
        },
    },
};
use dal_test::{
    WorkspaceSignup,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        create_unlocked_variant_copy_for_schema_name,
        encrypt_message,
    },
    test,
};
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;

mod action;
mod attribute;
mod authentication;

#[test]
#[ignore]
async fn get_bindings_for_latest_schema_variants(ctx: &mut DalContext) {
    let func_name = "test:createActionStarfield".to_string();

    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("found func")
        .expect("is some");

    let mut bindings = FuncBinding::for_func_id(ctx, func_id)
        .await
        .expect("found func bindings");
    assert_eq!(bindings.len(), 1);

    let binding = bindings.pop().expect("has a binding");

    let old_schema_variant_id = binding.get_schema_variant().expect("has a schema variant");

    // this schema variant is locked
    let old_schema_variant = SchemaVariant::get_by_id(ctx, old_schema_variant_id)
        .await
        .expect("has a schema variant");

    //this one is locked
    assert!(old_schema_variant.is_locked());

    let unlocked_binding = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
        .await
        .expect("got latest unlocked");
    // no unlocked bindings currently
    assert!(unlocked_binding.is_empty());

    // manually unlock the sv
    let new_sv = VariantAuthoringClient::create_unlocked_variant_copy(ctx, old_schema_variant_id)
        .await
        .expect("created unlocked copy");

    // new sv should have old funcs attached?!
    let new_bindings = FuncBinding::for_func_id(ctx, func_id)
        .await
        .expect("has bindings");

    assert_eq!(
        2,                  // expected
        new_bindings.len(), // actual
    );

    for binding in new_bindings {
        let _sv = SchemaVariant::get_by_id(ctx, binding.get_schema_variant().expect("has sv"))
            .await
            .expect("has sv");
    }

    // now we should have 1 unlocked func binding
    let mut latest_sv = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
        .await
        .expect("got latest for default");

    assert_eq!(1, latest_sv.len());

    let latest_sv_from_binding = latest_sv
        .pop()
        .expect("has one")
        .get_schema_variant()
        .expect("has sv");
    assert_eq!(latest_sv_from_binding, new_sv.id);

    let latest_unlocked = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
        .await
        .expect("got latest unlocked");

    // should have one latest unlocked now!
    assert_eq!(1, latest_unlocked.len());

    // now create a copy of the func (unlock it!)

    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("can create unlocked copy");
    let new_func_id = new_func.id;

    // get the bindings and make sure everything looks good
    let mut latest_sv = FuncBinding::get_bindings_for_default_schema_variants(ctx, new_func_id)
        .await
        .expect("got latest for default");

    assert_eq!(1, latest_sv.len());

    // latest sv should be the new one!
    let sv_id = latest_sv
        .pop()
        .expect("has one func")
        .get_schema_variant()
        .expect("has a schema variant");

    assert_eq!(new_sv.id, sv_id);

    // old func should have no unlocked variants
    let unlocked_binding = FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id)
        .await
        .expect("got latest unlocked");
    // no unlocked bindings currently
    assert!(unlocked_binding.is_empty());
}

#[test]
async fn for_action(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("no schema found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    let func_id = Func::find_id_by_name(ctx, "test:createActionSwifty")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");

    let bindings = FuncBinding::get_action_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get bindings")
        .pop()
        .expect("got one action binding");
    assert_eq!(
        ActionBinding {
            schema_variant_id,
            action_prototype_id: bindings.action_prototype_id,
            func_id,
            kind: ActionKind::Create
        },
        bindings
    );
}

#[test]
async fn for_qualification(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "dummy-secret")
        .await
        .expect("no schema found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    let func_id = Func::find_id_by_name(ctx, "test:qualificationDummySecretStringIsTodd")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");

    let binding = FuncBinding::get_qualification_bindings_for_func_id(ctx, func_id)
        .await
        .expect("got binding")
        .pop()
        .expect("has one binding");

    assert_eq!(
        LeafBinding {
            func_id,
            attribute_prototype_id: binding.attribute_prototype_id,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            inputs: vec![LeafInputLocation::Secrets],
            leaf_kind: LeafKind::Qualification
        },
        binding
    );
}

#[test]
async fn for_code_generation(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "katy perry")
        .await
        .expect("no schema found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    let func_id = Func::find_id_by_name(ctx, "test:generateStringCode")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let binding = FuncBinding::get_code_gen_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get leaf binding")
        .pop()
        .expect("could not get single binding");
    assert_eq!(
        LeafBinding {
            func_id,
            attribute_prototype_id: binding.attribute_prototype_id,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            inputs: vec![LeafInputLocation::Domain],
            leaf_kind: LeafKind::CodeGeneration
        },
        binding
    );
}

#[test]
async fn for_authentication(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "dummy-secret")
        .await
        .expect("no schema found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    let func_id = Func::find_id_by_name(ctx, "test:setDummySecretString")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let binding = FuncBinding::get_auth_bindings_for_func_id(ctx, func_id)
        .await
        .expect("got binding")
        .pop()
        .expect("got one binding");
    assert_eq!(
        AuthBinding {
            schema_variant_id,
            func_id
        },
        binding
    );
}

#[test]
async fn for_attribute_with_prop_input(ctx: &mut DalContext) {
    let func_id = Func::find_id_by_name(ctx, "hesperus_is_phosphorus")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");

    let binding = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get binding")
        .pop()
        .expect("got the binding");

    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("no schema found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    // Find the sole func argument id. Ensure there is only one.
    let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
        .await
        .expect("could not list func argument ids");
    let func_argument_id = func_argument_ids
        .pop()
        .expect("func argument ids are empty");
    assert!(func_argument_ids.is_empty());

    // Find the sole attribute prototype  id. Ensure there is only one.
    let mut attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list attribute prototype ids");
    let attribute_prototype_id = attribute_prototype_ids
        .pop()
        .expect("attribute prototype ids are empty");
    assert!(attribute_prototype_ids.is_empty());

    // Find the sole attribute prototype argument id. Ensure there is only one.
    let mut attribute_prototype_argument_ids =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
            .await
            .expect("could not list attribute prototype argument ids");
    let attribute_prototype_argument_id = attribute_prototype_argument_ids
        .pop()
        .expect("attribute prototype argument ids are empty");
    assert!(attribute_prototype_argument_ids.is_empty());

    // Find both props.
    let output_location_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new([
            "root",
            "domain",
            "possible_world_b",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "naming_and_necessity",
        ]),
    )
    .await
    .expect("could not find prop by path");
    let prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new([
            "root",
            "domain",
            "possible_world_a",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "rigid_designator",
        ]),
    )
    .await
    .expect("could not find prop by path");

    assert_eq!(
        AttributeBinding {
            func_id,
            attribute_prototype_id,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            output_location: AttributeFuncDestination::Prop(output_location_prop_id),
            argument_bindings: vec![AttributeArgumentBinding {
                func_argument_id,
                attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
                attribute_func_input_location: AttributeFuncArgumentSource::Prop(prop_id)
            }]
        },
        binding
    );

    let mut func_arguments = FuncArgument::list_for_func(ctx, func_id)
        .await
        .expect("could not list func arguments");
    let func_argument = func_arguments.pop().expect("empty func arguments");
    assert!(func_arguments.is_empty());
    assert_eq!(func_argument_id, func_argument.id);
    assert_eq!("hesperus", func_argument.name.as_str());
    assert_eq!(FuncArgumentKind::String, func_argument.kind);
    assert_eq!(None, func_argument.element_kind);
}

#[test]
async fn for_attribute_with_input_socket_input(ctx: &mut DalContext) {
    let func_id = Func::find_id_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");

    let binding = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get binding")
        .pop()
        .expect("got the binding");

    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("no schema found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("could not perform get default schema variant");

    // Find the sole func argument id. Ensure there is only one.
    let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id)
        .await
        .expect("could not list func argument ids");
    let func_argument_id = func_argument_ids
        .pop()
        .expect("func argument ids are empty");
    assert!(func_argument_ids.is_empty());

    // Find the sole attribute prototype  id. Ensure there is only one.
    let mut attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list attribute prototype ids");
    let attribute_prototype_id = attribute_prototype_ids
        .pop()
        .expect("attribute prototype ids are empty");
    assert!(attribute_prototype_ids.is_empty());

    // Find the sole attribute prototype argument id. Ensure there is only one.
    let mut attribute_prototype_argument_ids =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id)
            .await
            .expect("could not list attribute prototype argument ids");
    let attribute_prototype_argument_id = attribute_prototype_argument_ids
        .pop()
        .expect("attribute prototype argument ids are empty");
    assert!(attribute_prototype_argument_ids.is_empty());

    // Find the sole input socket id. Ensure there is only one.
    let mut input_socket_ids =
        AttributePrototype::list_input_socket_sources_for_id(ctx, attribute_prototype_id)
            .await
            .expect("could not list input socket ids");
    let input_socket_id = input_socket_ids.pop().expect("input socket ids are empty");
    assert!(input_socket_ids.is_empty());

    // Find the prop for the prototype.
    let prop = Prop::find_prop_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "universe", "galaxies"]),
    )
    .await
    .expect("could not find prop by path");

    assert_eq!(
        AttributeBinding {
            func_id,
            attribute_prototype_id,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            output_location: AttributeFuncDestination::Prop(prop.id),
            argument_bindings: vec![AttributeArgumentBinding {
                func_argument_id,
                attribute_prototype_argument_id: Some(attribute_prototype_argument_id),
                attribute_func_input_location: AttributeFuncArgumentSource::InputSocket(
                    input_socket_id
                )
            }]
        },
        binding
    );

    let mut func_arguments = FuncArgument::list_for_func(ctx, func_id)
        .await
        .expect("could not list func arguments");
    let func_argument = func_arguments.pop().expect("empty func arguments");
    assert!(func_arguments.is_empty());
    assert_eq!(func_argument_id, func_argument.id);
    assert_eq!("entries", func_argument.name.as_str());
    assert_eq!(FuncArgumentKind::Array, func_argument.kind);
    assert_eq!(Some(FuncArgumentKind::Object), func_argument.element_kind);
}

#[test]
async fn for_intrinsics(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get schema variant");
    let all_funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("unable to get all funcs");
    let mut unset_props = Vec::from([
        PropPath::new(["root"]),
        PropPath::new(["root", "si", "protected"]),
        PropPath::new(["root", "si"]),
        PropPath::new(["root", "si", "protected"]),
        PropPath::new(["root", "si", "resourceId"]),
        PropPath::new(["root", "secrets"]),
        PropPath::new(["root", "resource"]),
        PropPath::new(["root", "resource", "message"]),
        PropPath::new(["root", "resource", "last_synced"]),
        PropPath::new(["root", "resource", "payload"]),
        PropPath::new(["root", "resource", "status"]),
        PropPath::new(["root", "code"]),
        PropPath::new(["root", "code", "codeItem"]),
        PropPath::new(["root", "code", "codeItem", "format"]),
        PropPath::new(["root", "code", "codeItem", "code"]),
        PropPath::new(["root", "qualification"]),
        PropPath::new(["root", "qualification", "qualificationItem"]),
        PropPath::new(["root", "qualification", "qualificationItem", "result"]),
        PropPath::new(["root", "qualification", "qualificationItem", "message"]),
        PropPath::new(["root", "domain", "universe"]),
        PropPath::new(["root", "domain"]),
        PropPath::new(["root", "domain", "universe", "galaxies", "galaxy"]),
        PropPath::new([
            "root", "domain", "universe", "galaxies", "galaxy", "planets",
        ]),
        PropPath::new(["root", "domain", "universe", "galaxies", "galaxy", "sun"]),
        PropPath::new(["root", "domain", "possible_world_b"]),
        PropPath::new(["root", "domain", "possible_world_b", "wormhole_1"]),
        PropPath::new([
            "root",
            "domain",
            "possible_world_b",
            "wormhole_1",
            "wormhole_2",
        ]),
        PropPath::new([
            "root",
            "domain",
            "possible_world_b",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
        ]),
        PropPath::new([
            "root",
            "domain",
            "possible_world_a",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "rigid_designator",
        ]),
        PropPath::new(["root", "domain", "possible_world_a"]),
        PropPath::new(["root", "domain", "possible_world_a", "wormhole_1"]),
        PropPath::new([
            "root",
            "domain",
            "possible_world_a",
            "wormhole_1",
            "wormhole_2",
        ]),
        PropPath::new([
            "root",
            "domain",
            "possible_world_a",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
        ]),
        PropPath::new(["root", "domain", "freestar"]),
        PropPath::new(["root", "domain", "hidden_prop"]),
        PropPath::new(["root", "deleted_at"]),
    ]);
    for func in all_funcs {
        match func.backend_kind {
            dal::FuncBackendKind::Unset => {
                let attribute_bindings: Vec<AttributeBinding> =
                    FuncBinding::get_attribute_bindings_for_func_id(ctx, func.id)
                        .await
                        .expect("could not get attribute bindings")
                        .into_iter()
                        .filter(|binding| {
                            binding.eventual_parent
                                == EventualParent::SchemaVariant(schema_variant_id)
                        })
                        .collect();
                let mut prop_paths_for_bindings: Vec<PropPath> = vec![];
                for binding in attribute_bindings {
                    let AttributeFuncDestination::Prop(prop_id) = binding.output_location else {
                        panic!("Non-Prop is set to unset, which is unexpected!")
                    };
                    let prop = Prop::get_by_id(ctx, prop_id)
                        .await
                        .expect("couldn't get prop");
                    let path = prop.path(ctx).await.expect("could not get prop path");
                    prop_paths_for_bindings.push(path);
                }
                // prop_paths_for_bindings.retain(|binding| !unset_props.contains(binding));
                prop_paths_for_bindings.retain(|prop_path| {
                    if unset_props.contains(prop_path) {
                        unset_props.retain(|p| p != prop_path);
                        false
                    } else {
                        true
                    }
                });
                assert!(prop_paths_for_bindings.is_empty());
            }
            dal::FuncBackendKind::Identity => {
                let attribute_bindings: Vec<AttributeBinding> =
                    FuncBinding::get_attribute_bindings_for_func_id(ctx, func.id)
                        .await
                        .expect("could not get attribute bindings")
                        .into_iter()
                        .filter(|binding| {
                            binding.eventual_parent
                                == EventualParent::SchemaVariant(schema_variant_id)
                        })
                        .collect();
                assert_eq!(2, attribute_bindings.len());
                for binding in attribute_bindings {
                    if let AttributeFuncDestination::Prop(prop_id) = binding.output_location {
                        let prop = Prop::get_by_id(ctx, prop_id)
                            .await
                            .expect("couldn't get prop");
                        let path = prop.path(ctx).await.expect("bad");
                        match path.with_replaced_sep("/").as_str() {
                            "root/domain/name" => {
                                let arg_bindings: AttributeFuncArgumentSource = binding
                                    .clone()
                                    .argument_bindings
                                    .into_iter()
                                    .map(|binding| binding.attribute_func_input_location)
                                    .collect_vec()
                                    .pop()
                                    .expect("has a value");
                                let AttributeFuncArgumentSource::Prop(prop_id) = arg_bindings
                                else {
                                    panic!("Non-Prop is set to unset, which is unexpected!")
                                };
                                let prop = Prop::get_by_id(ctx, prop_id)
                                    .await
                                    .expect("couldn't get prop");
                                let path = prop
                                    .path(ctx)
                                    .await
                                    .expect("couldn't get prop path")
                                    .with_replaced_sep("/");
                                // ensure root/domain/name takes its value from root/si/name
                                assert_eq!("root/si/name", path);
                            }
                            "root/domain/attributes" => {
                                let arg_bindings: AttributeFuncArgumentSource = binding
                                    .clone()
                                    .argument_bindings
                                    .into_iter()
                                    .map(|binding| binding.attribute_func_input_location)
                                    .collect_vec()
                                    .pop()
                                    .expect("has a value");
                                let AttributeFuncArgumentSource::InputSocket(input_socket_id) =
                                    arg_bindings
                                else {
                                    panic!("Non-Prop is set to unset, which is unexpected!")
                                };
                                let input_socket = InputSocket::get_by_id(ctx, input_socket_id)
                                    .await
                                    .expect("couldn't get prop");
                                // ensure root/domain/attributes takes its value from the bethesda socket
                                assert_eq!("bethesda", input_socket.name())
                            }
                            _ => panic!("unexpected prop set to identity"),
                        }
                    };
                    if let AttributeFuncDestination::InputSocket(_) = binding.output_location {
                        panic!("unexpected input socket set to identity")
                    }
                }
            }
            // NOTE(nick): it would be ideal to address resourcePayloadToValue and normalizeToArray.
            dal::FuncBackendKind::ResourcePayloadToValue
            | dal::FuncBackendKind::NormalizeToArray
            | dal::FuncBackendKind::JsAttribute
            | dal::FuncBackendKind::JsAction => {} // not testing these right now
            _ => {
                panic!("there should not be any other funcs returned for this variant");
            }
        }
    }
    // make sure we found all of the expected props/sockets associated with intrinsics we care about (unset + identity)
    assert!(unset_props.is_empty());
}

#[test]
async fn code_gen_cannot_create_cycle(ctx: &mut DalContext) {
    let _schema = Schema::get_by_name(ctx, "katy perry")
        .await
        .expect("no schema found");
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "katy perry")
        .await
        .expect("could not create unlocked variant copy");

    let func_id = Func::find_id_by_name(ctx, "test:generateStringCode")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let binding = FuncBinding::get_code_gen_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get leaf binding")
        .into_iter()
        .find(|binding| binding.eventual_parent == EventualParent::SchemaVariant(schema_variant_id))
        .expect("bang");
    assert_eq!(
        LeafBinding {
            func_id,
            attribute_prototype_id: binding.attribute_prototype_id,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            inputs: vec![LeafInputLocation::Domain],
            leaf_kind: LeafKind::CodeGeneration
        },
        binding
    );
    let _cycle_check_guard = ctx
        .workspace_snapshot()
        .expect("got snap")
        .enable_cycle_check()
        .await;
    let result = LeafBinding::update_leaf_func_binding(
        ctx,
        binding.attribute_prototype_id,
        &[LeafInputLocation::Domain, LeafInputLocation::Code],
    )
    .await;

    match result {
        Err(err) if err.is_create_graph_cycle() => {}
        other => panic!("Test should fail if we don't get this error, got: {other:?}"),
    }
}

#[test]
async fn return_the_right_bindings(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    // create two components and draw an edge between them
    // one is a secret defining component
    // create a complicated component too
    // ensure we're returning the right data
    let _starfield =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", "starfield")
            .await
            .expect("could not create component");
    let source_component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "source")
            .await
            .expect("could not create component");
    let source_schema_variant_id = Component::schema_variant_id(ctx, source_component.id())
        .await
        .expect("could not get schema variant id for component");
    let destination_component =
        create_component_for_default_schema_name_in_default_view(ctx, "fallout", "destination")
            .await
            .expect("could not create component");
    let destination_schema_variant_id =
        Component::schema_variant_id(ctx, destination_component.id())
            .await
            .expect("could not get schema variant id for component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Cache the name of the secret definition from the test exclusive schema. Afterward, cache the
    // prop we need for attribute value update.
    let secret_definition_name = "dummy";
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        source_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // Connect the two components to propagate the secret value and commit.
    let source_output_socket = OutputSocket::find_with_name(ctx, "dummy", source_schema_variant_id)
        .await
        .expect("could not perform find with name")
        .expect("output socket not found by name");
    let destination_input_socket =
        InputSocket::find_with_name(ctx, "dummy", destination_schema_variant_id)
            .await
            .expect("could not perform find with name")
            .expect("input socket not found by name");
    Component::connect(
        ctx,
        source_component.id(),
        source_output_socket.id(),
        destination_component.id(),
        destination_input_socket.id(),
    )
    .await
    .expect("could not connect");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create the secret and commit.
    let secret = Secret::new(
        ctx,
        "johnqt",
        secret_definition_name.to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message"),
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Use the secret in the source component and commit.
    let property_values = PropertyEditorValues::assemble(ctx, source_component.id())
        .await
        .expect("unable to list prop values");
    let reference_to_secret_attribute_value_id = property_values
        .find_by_prop_id(reference_to_secret_prop.id)
        .expect("could not find attribute value");
    Secret::attach_for_attribute_value(
        ctx,
        reference_to_secret_attribute_value_id,
        Some(secret.id()),
    )
    .await
    .expect("could not attach secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // now lets get bindings and ensure nothing explodes
    let funcs = Func::list_all(ctx).await.expect("could not get funcs");
    for func in funcs {
        let bindings = FuncBinding::for_func_id(ctx, func.id)
            .await
            .expect("could not get func bindings");
        let intrinsic = Func::get_intrinsic_kind_by_id_or_error(ctx, func.id).await;
        let maybe_intrinsic = intrinsic.ok();

        for binding in bindings {
            match binding {
                FuncBinding::Attribute(attribute_binding) => {
                    match attribute_binding.output_location {
                        AttributeFuncDestination::Prop(_) => {
                            // if the output location is a prop and the func is intrinsic, there are special things
                            if let Some(intrinsic) = maybe_intrinsic {
                                match intrinsic {
                                    IntrinsicFunc::Identity
                                    | IntrinsicFunc::NormalizeToArray
                                    | IntrinsicFunc::ResourcePayloadToValue => {
                                        // Props only take inputs from input sockets or other props
                                        let mut maybe_invalid_args =
                                            attribute_binding.argument_bindings.clone();
                                        let mut maybe_valid_args =
                                            attribute_binding.argument_bindings.clone();
                                        maybe_invalid_args.retain(|arg| {
                                            match arg.attribute_func_input_location {
                                                AttributeFuncArgumentSource::Prop(_) => false,
                                                AttributeFuncArgumentSource::InputSocket(_) => {
                                                    false
                                                }
                                                AttributeFuncArgumentSource::StaticArgument(_) => {
                                                    false // is this allowed? todo
                                                }
                                                AttributeFuncArgumentSource::OutputSocket(_) => {
                                                    true
                                                }
                                                AttributeFuncArgumentSource::Secret(_) => true,
                                            }
                                        });
                                        assert!(maybe_invalid_args.is_empty());
                                        maybe_valid_args.retain(|arg| {
                                            match arg.attribute_func_input_location {
                                                AttributeFuncArgumentSource::Prop(_) => true,
                                                AttributeFuncArgumentSource::InputSocket(_) => true,
                                                AttributeFuncArgumentSource::StaticArgument(_) => {
                                                    false
                                                }
                                                AttributeFuncArgumentSource::OutputSocket(_) => {
                                                    false
                                                }
                                                AttributeFuncArgumentSource::Secret(_) => false,
                                            }
                                        });
                                        // should only be one input right now
                                        assert_eq!(maybe_valid_args.len(), 1);
                                    }
                                    IntrinsicFunc::Unset => {
                                        assert!(attribute_binding.argument_bindings.is_empty());
                                    }
                                    IntrinsicFunc::SetArray
                                    | IntrinsicFunc::SetBoolean
                                    | IntrinsicFunc::SetInteger
                                    | IntrinsicFunc::SetJson
                                    | IntrinsicFunc::SetFloat
                                    | IntrinsicFunc::SetMap
                                    | IntrinsicFunc::SetObject
                                    | IntrinsicFunc::SetString
                                    | IntrinsicFunc::Validation => {
                                        assert_eq!(attribute_binding.argument_bindings.len(), 1);
                                    }
                                }
                            }
                        }
                        AttributeFuncDestination::OutputSocket(_) => {
                            if let Some(intrinsic) = maybe_intrinsic {
                                match intrinsic {
                                    IntrinsicFunc::Identity
                                    | IntrinsicFunc::NormalizeToArray
                                    | IntrinsicFunc::ResourcePayloadToValue => {
                                        // Output Sockets only take inputs from input sockets or props
                                        let mut maybe_invalid_args =
                                            attribute_binding.argument_bindings.clone();
                                        let mut maybe_valid_args =
                                            attribute_binding.argument_bindings.clone();
                                        maybe_invalid_args.retain(|arg| {
                                            match arg.attribute_func_input_location {
                                                AttributeFuncArgumentSource::Prop(_) => false,
                                                AttributeFuncArgumentSource::InputSocket(_) => {
                                                    false
                                                }
                                                AttributeFuncArgumentSource::StaticArgument(_) => {
                                                    true
                                                }
                                                AttributeFuncArgumentSource::OutputSocket(_) => {
                                                    true
                                                }
                                                AttributeFuncArgumentSource::Secret(_) => true,
                                            }
                                        });
                                        assert!(maybe_invalid_args.is_empty());
                                        maybe_valid_args.retain(|arg| {
                                            match arg.attribute_func_input_location {
                                                AttributeFuncArgumentSource::Prop(_) => true,
                                                AttributeFuncArgumentSource::InputSocket(_) => true,
                                                AttributeFuncArgumentSource::StaticArgument(_) => {
                                                    false
                                                }
                                                AttributeFuncArgumentSource::OutputSocket(_) => {
                                                    false
                                                }
                                                AttributeFuncArgumentSource::Secret(_) => false,
                                            }
                                        });
                                        // should only be one or zero input right now
                                        assert!(maybe_valid_args.len() < 2);
                                    }
                                    // unset has no args
                                    IntrinsicFunc::Unset => {
                                        assert!(attribute_binding.argument_bindings.is_empty());
                                    }
                                    // these intrinsics only have one arg
                                    IntrinsicFunc::SetArray
                                    | IntrinsicFunc::SetBoolean
                                    | IntrinsicFunc::SetFloat
                                    | IntrinsicFunc::SetInteger
                                    | IntrinsicFunc::SetJson
                                    | IntrinsicFunc::SetMap
                                    | IntrinsicFunc::SetObject
                                    | IntrinsicFunc::SetString
                                    | IntrinsicFunc::Validation => {
                                        assert_eq!(attribute_binding.argument_bindings.len(), 1);
                                    }
                                }
                            }
                        }

                        AttributeFuncDestination::InputSocket(_) => {
                            panic!("should not be seeing input sockets for output locations")
                        }
                    }
                }

                FuncBinding::Authentication(_)
                | FuncBinding::Management(_)
                | FuncBinding::Action(_)
                | FuncBinding::CodeGeneration(_)
                | FuncBinding::Qualification(_) => {} // nothing really to check here
            }
        }

        let func_summary = func
            .into_frontend_type(ctx)
            .await
            .expect("could not get front end type");
        for binding in func_summary.bindings {
            match binding {
                si_frontend_types::FuncBinding::Action {
                    schema_variant_id,
                    action_prototype_id,
                    func_id,
                    kind,
                } => {
                    assert!(kind.is_some());
                    assert!(schema_variant_id.is_some());
                    assert!(func_id.is_some());
                    assert!(action_prototype_id.is_some());
                }
                si_frontend_types::FuncBinding::Attribute {
                    component_id,
                    schema_variant_id,
                    prop_id,
                    output_socket_id,
                    func_id,
                    attribute_prototype_id,
                    ..
                } => {
                    assert!(component_id.is_none());
                    assert!(schema_variant_id.is_some());
                    match prop_id {
                        Some(_) => {
                            assert!(output_socket_id.is_none());
                        }
                        None => assert!(output_socket_id.is_some()),
                    }

                    assert!(func_id.is_some());
                    assert!(attribute_prototype_id.is_some());
                }
                si_frontend_types::FuncBinding::CodeGeneration {
                    schema_variant_id,
                    component_id,
                    inputs,
                    func_id,
                    attribute_prototype_id,
                } => {
                    assert!(schema_variant_id.is_some());
                    assert!(component_id.is_none());
                    for input in inputs {
                        assert!(input != LeafInputLocation::Code.into())
                    }
                    assert!(func_id.is_some());
                    assert!(attribute_prototype_id.is_some());
                }
                si_frontend_types::FuncBinding::Qualification {
                    schema_variant_id,
                    component_id,
                    func_id,
                    attribute_prototype_id,
                    ..
                } => {
                    assert!(schema_variant_id.is_some());
                    assert!(component_id.is_none());
                    assert!(func_id.is_some());
                    assert!(attribute_prototype_id.is_some());
                }
                si_frontend_types::FuncBinding::Authentication { func_id, .. } => {
                    assert!(func_id.is_some());
                }
                si_frontend_types::FuncBinding::Management {
                    schema_variant_id,
                    management_prototype_id,
                    func_id,
                    ..
                } => {
                    assert!(schema_variant_id.is_some());
                    assert!(func_id.is_some());
                    assert!(management_prototype_id.is_some());
                }
            }
        }
    }
}
