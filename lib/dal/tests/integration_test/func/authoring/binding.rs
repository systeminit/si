use dal::{
    AttributePrototype,
    Component,
    DalContext,
    Func,
    InputSocket,
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
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
    },
    prop::PropPath,
    property_editor::values::PropertyEditorValues,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    WorkspaceSignup,
    helpers::{
        attribute::value,
        change_set,
        component,
        create_component_for_default_schema_name_in_default_view,
        create_unlocked_variant_copy_for_schema_name,
        encrypt_message,
        func,
        schema::variant,
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
async fn get_bindings_for_latest_schema_variants(ctx: &mut DalContext) -> Result<()> {
    let func_id = func::id(ctx, "test:createActionStarfield").await?;

    let mut bindings = FuncBinding::for_func_id(ctx, func_id).await?;
    assert_eq!(bindings.len(), 1);

    let binding = bindings.pop().expect("has a binding");

    let old_schema_variant_id = binding.get_schema_variant().expect("has a schema variant");

    // this schema variant is locked
    let old_schema_variant = SchemaVariant::get_by_id(ctx, old_schema_variant_id).await?;

    //this one is locked
    assert!(old_schema_variant.is_locked());

    let unlocked_binding =
        FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;
    // no unlocked bindings currently
    assert!(unlocked_binding.is_empty());

    // manually unlock the sv
    let new_sv =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, old_schema_variant_id).await?;

    // new sv should have old funcs attached?!
    let new_bindings = FuncBinding::for_func_id(ctx, func_id).await?;

    assert_eq!(
        2,                  // expected
        new_bindings.len(), // actual
    );

    for binding in new_bindings {
        let _sv =
            SchemaVariant::get_by_id(ctx, binding.get_schema_variant().expect("has sv")).await?;
    }

    // now we should have 1 unlocked func binding
    let mut latest_sv =
        FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;

    assert_eq!(1, latest_sv.len());

    let latest_sv_from_binding = latest_sv
        .pop()
        .expect("has one")
        .get_schema_variant()
        .expect("has sv");
    assert_eq!(latest_sv_from_binding, new_sv.id);

    let latest_unlocked =
        FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;

    // should have one latest unlocked now!
    assert_eq!(1, latest_unlocked.len());

    // now create a copy of the func (unlock it!)

    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None).await?;
    let new_func_id = new_func.id;

    // get the bindings and make sure everything looks good
    let mut latest_sv =
        FuncBinding::get_bindings_for_default_schema_variants(ctx, new_func_id).await?;

    assert_eq!(1, latest_sv.len());

    // latest sv should be the new one!
    let sv_id = latest_sv
        .pop()
        .expect("has one func")
        .get_schema_variant()
        .expect("has a schema variant");

    assert_eq!(new_sv.id, sv_id);

    // old func should have no unlocked variants
    let unlocked_binding =
        FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;
    // no unlocked bindings currently
    assert!(unlocked_binding.is_empty());

    Ok(())
}

#[test]
async fn for_action(ctx: &mut DalContext) -> Result<()> {
    let schema_variant_id = variant::id(ctx, "swifty").await?;
    let func_id = func::id(ctx, "test:createActionSwifty").await?;

    let bindings = FuncBinding::get_action_bindings_for_func_id(ctx, func_id)
        .await?
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

    Ok(())
}

#[test]
async fn for_qualification(ctx: &mut DalContext) -> Result<()> {
    let schema_variant_id = variant::id(ctx, "dummy-secret").await?;
    let func_id = func::id(ctx, "test:qualificationDummySecretStringIsTodd").await?;

    let binding = FuncBinding::get_qualification_bindings_for_func_id(ctx, func_id)
        .await?
        .pop()
        .expect("has one binding");

    assert_eq!(
        LeafBinding {
            func_id,
            leaf_binding_prototype: binding.leaf_binding_prototype,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            inputs: vec![LeafInputLocation::Secrets],
            leaf_kind: LeafKind::Qualification
        },
        binding
    );

    Ok(())
}

#[test]
async fn for_code_generation(ctx: &mut DalContext) -> Result<()> {
    let schema_variant_id = variant::id(ctx, "katy perry").await?;

    let func_id = func::id(ctx, "test:generateStringCode").await?;
    let binding = FuncBinding::get_code_gen_bindings_for_func_id(ctx, func_id)
        .await?
        .pop()
        .expect("could not get single binding");
    assert_eq!(
        LeafBinding {
            func_id,
            leaf_binding_prototype: binding.leaf_binding_prototype,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            inputs: vec![LeafInputLocation::Domain],
            leaf_kind: LeafKind::CodeGeneration
        },
        binding
    );

    Ok(())
}

#[test]
async fn for_authentication(ctx: &mut DalContext) -> Result<()> {
    let schema_variant_id = variant::id(ctx, "dummy-secret").await?;
    let func_id = func::id(ctx, "test:setDummySecretString").await?;

    let binding = FuncBinding::get_auth_bindings_for_func_id(ctx, func_id)
        .await?
        .pop()
        .expect("got one binding");
    assert_eq!(
        AuthBinding {
            schema_variant_id,
            func_id
        },
        binding
    );

    Ok(())
}

#[test]
async fn for_attribute_with_prop_input(ctx: &mut DalContext) -> Result<()> {
    let func_id = func::id(ctx, "hesperus_is_phosphorus").await?;

    let binding = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await?
        .pop()
        .expect("got the binding");

    let schema_variant_id = variant::id(ctx, "starfield").await?;

    // Find the sole func argument id. Ensure there is only one.
    let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id).await?;
    let func_argument_id = func_argument_ids
        .pop()
        .expect("func argument ids are empty");
    assert!(func_argument_ids.is_empty());

    // Find the sole attribute prototype  id. Ensure there is only one.
    let mut attribute_prototype_ids =
        AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
    let attribute_prototype_id = attribute_prototype_ids
        .pop()
        .expect("attribute prototype ids are empty");
    assert!(attribute_prototype_ids.is_empty());

    // Find the sole attribute prototype argument id. Ensure there is only one.
    let mut attribute_prototype_argument_ids =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;
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
    .await?;
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
    .await?;

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

    let mut func_arguments = FuncArgument::list_for_func(ctx, func_id).await?;
    let func_argument = func_arguments.pop().expect("empty func arguments");
    assert!(func_arguments.is_empty());
    assert_eq!(func_argument_id, func_argument.id);
    assert_eq!("hesperus", func_argument.name.as_str());
    assert_eq!(FuncArgumentKind::String, func_argument.kind);
    assert_eq!(None, func_argument.element_kind);

    Ok(())
}

#[test]
async fn for_subscription(ctx: &mut DalContext) -> Result<()> {
    // Create a subscription using the given attribute function
    component::create(ctx, "fallout", "source").await?;
    component::create(ctx, "fallout", "destination").await?;
    value::subscribe_with_custom_function(
        ctx,
        ("destination", "/si/name"),
        ("source", "/si/name"),
        "test:falloutEntriesToGalaxies",
    )
    .await?;
    change_set::commit(ctx).await?;

    // Make sure the subscription binding converts to the frontend type
    let func_id = func::id(ctx, "test:falloutEntriesToGalaxies").await?;
    FuncBinding::for_func_id(ctx, func_id)
        .await
        .expect("could not get func bindings");

    Ok(())
}

#[test]
async fn for_attribute_with_input_socket_input(ctx: &mut DalContext) -> Result<()> {
    let func_id = func::id(ctx, "test:falloutEntriesToGalaxies").await?;
    let schema_variant_id = variant::id(ctx, "starfield").await?;

    let binding = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await?
        .pop()
        .expect("got the binding");

    // Find the sole func argument id. Ensure there is only one.
    let mut func_argument_ids = FuncArgument::list_ids_for_func(ctx, func_id).await?;
    let func_argument_id = func_argument_ids
        .pop()
        .expect("func argument ids are empty");
    assert!(func_argument_ids.is_empty());

    // Find the sole attribute prototype  id. Ensure there is only one.
    let mut attribute_prototype_ids =
        AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
    let attribute_prototype_id = attribute_prototype_ids
        .pop()
        .expect("attribute prototype ids are empty");
    assert!(attribute_prototype_ids.is_empty());

    // Find the sole attribute prototype argument id. Ensure there is only one.
    let mut attribute_prototype_argument_ids =
        AttributePrototypeArgument::list_ids_for_prototype(ctx, attribute_prototype_id).await?;
    let attribute_prototype_argument_id = attribute_prototype_argument_ids
        .pop()
        .expect("attribute prototype argument ids are empty");
    assert!(attribute_prototype_argument_ids.is_empty());

    // Find the sole input socket id. Ensure there is only one.
    let mut input_socket_ids =
        AttributePrototype::list_input_socket_sources_for_id(ctx, attribute_prototype_id).await?;
    let input_socket_id = input_socket_ids.pop().expect("input socket ids are empty");
    assert!(input_socket_ids.is_empty());

    // Find the prop for the prototype.
    let prop = Prop::find_prop_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "universe", "galaxies"]),
    )
    .await?;

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

    let mut func_arguments = FuncArgument::list_for_func(ctx, func_id).await?;
    let func_argument = func_arguments.pop().expect("empty func arguments");
    assert!(func_arguments.is_empty());
    assert_eq!(func_argument_id, func_argument.id);
    assert_eq!("entries", func_argument.name.as_str());
    assert_eq!(FuncArgumentKind::Array, func_argument.kind);
    assert_eq!(Some(FuncArgumentKind::Object), func_argument.element_kind);

    Ok(())
}

#[test]
async fn for_intrinsics(ctx: &mut DalContext) -> Result<()> {
    let schema_variant_id = variant::id(ctx, "starfield").await?;
    let all_funcs = SchemaVariant::all_funcs(ctx, schema_variant_id).await?;
    let mut unset_props = Vec::from([
        PropPath::new(["root"]),
        PropPath::new(["root", "si", "protected"]),
        PropPath::new(["root", "si"]),
        PropPath::new(["root", "si", "protected"]),
        PropPath::new(["root", "si", "resourceId"]),
        PropPath::new(["root", "si", "tags"]),
        PropPath::new(["root", "si", "tags", "tag"]),
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
                        .await?
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
                    let prop = Prop::get_by_id(ctx, prop_id).await?;
                    let path = prop.path(ctx).await?;
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
                        .await?
                        .into_iter()
                        .filter(|binding| {
                            binding.eventual_parent
                                == EventualParent::SchemaVariant(schema_variant_id)
                        })
                        .collect();
                assert_eq!(2, attribute_bindings.len());
                for binding in attribute_bindings {
                    if let AttributeFuncDestination::Prop(prop_id) = binding.output_location {
                        let prop = Prop::get_by_id(ctx, prop_id).await?;
                        let path = prop.path(ctx).await?;
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
                                let prop = Prop::get_by_id(ctx, prop_id).await?;
                                let path = prop.path(ctx).await?.with_replaced_sep("/");
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
                                let input_socket =
                                    InputSocket::get_by_id(ctx, input_socket_id).await?;
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

    Ok(())
}

#[test]
async fn code_gen_cannot_create_cycle(ctx: &mut DalContext) -> Result<()> {
    let _schema = Schema::get_by_name(ctx, "katy perry").await?;
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "katy perry").await?;

    let func_id = func::id(ctx, "test:generateStringCode").await?;
    let binding = FuncBinding::get_code_gen_bindings_for_func_id(ctx, func_id)
        .await?
        .into_iter()
        .find(|binding| binding.eventual_parent == EventualParent::SchemaVariant(schema_variant_id))
        .expect("bang");
    assert_eq!(
        LeafBinding {
            func_id,
            leaf_binding_prototype: binding.leaf_binding_prototype,
            eventual_parent: EventualParent::SchemaVariant(schema_variant_id),
            inputs: vec![LeafInputLocation::Domain],
            leaf_kind: LeafKind::CodeGeneration
        },
        binding
    );
    let _cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
    let result = LeafBinding::update_leaf_func_binding(
        ctx,
        binding.leaf_binding_prototype,
        &[LeafInputLocation::Domain, LeafInputLocation::Code],
    )
    .await;

    match result {
        Err(err) if err.is_create_graph_cycle() => {}
        other => panic!("Test should fail if we don't get this error, got: {other:?}"),
    }

    Ok(())
}

#[test]
async fn return_the_right_bindings(ctx: &mut DalContext, nw: &WorkspaceSignup) -> Result<()> {
    // create two components and draw an edge between them
    // one is a secret defining component
    // create a complicated component too
    // ensure we're returning the right data
    let _starfield =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", "starfield")
            .await?;
    let source_component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "source")
            .await?;
    let source_schema_variant_id = Component::schema_variant_id(ctx, source_component.id()).await?;
    let _destination_component =
        create_component_for_default_schema_name_in_default_view(ctx, "fallout", "destination")
            .await?;
    change_set::commit(ctx).await?;

    // Cache the name of the secret definition from the test exclusive schema. Afterward, cache the
    // prop we need for attribute value update.
    let secret_definition_name = "dummy";
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        source_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await?;

    // Connect the two components to propagate the secret value and commit.
    value::subscribe(
        ctx,
        ("destination", "/secrets/dummy"),
        ("source", "/secrets/dummy"),
    )
    .await?;
    change_set::commit(ctx).await?;

    // Create the secret and commit.
    let secret = Secret::new(
        ctx,
        "johnqt",
        secret_definition_name.to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}]).await?,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await?;
    change_set::commit(ctx).await?;

    // Use the secret in the source component and commit.
    let property_values = PropertyEditorValues::assemble(ctx, source_component.id()).await?;
    let reference_to_secret_attribute_value_id = property_values
        .find_by_prop_id(reference_to_secret_prop.id)
        .expect("could not find attribute value");
    Secret::attach_for_attribute_value(
        ctx,
        reference_to_secret_attribute_value_id,
        Some(secret.id()),
    )
    .await?;
    change_set::commit(ctx).await?;

    // Connect the two components with a custom function to make sure the bindings don't get returned
    value::subscribe_with_custom_function(
        ctx,
        ("destination", "/si/name"),
        ("source", "/si/name"),
        "test:falloutEntriesToGalaxies",
    )
    .await?;

    // now lets get bindings and ensure nothing explodes
    let funcs = Func::list_all(ctx).await?;
    for func in funcs {
        let bindings = FuncBinding::for_func_id(ctx, func.id).await?;
        let intrinsic = Func::intrinsic_kind_or_error(ctx, func.id).await;
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
                                                AttributeFuncArgumentSource::ValueSubscription { .. } => false,
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
                                                AttributeFuncArgumentSource::ValueSubscription { .. } => false,
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
                                                AttributeFuncArgumentSource::ValueSubscription { .. } => false,
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
                                                AttributeFuncArgumentSource::ValueSubscription { .. } => false,
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

        let func_summary = func.into_frontend_type(ctx).await?;
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
                    // It must either have a component_id or a schema_variant_id, but not both.
                    assert!(component_id.is_some() != schema_variant_id.is_some());
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
                    ..
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

    Ok(())
}
