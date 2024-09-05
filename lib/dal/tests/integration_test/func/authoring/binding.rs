use dal::{
    action::prototype::ActionKind,
    attribute::prototype::argument::{AttributePrototypeArgument, AttributePrototypeArgumentError},
    func::{
        argument::{FuncArgument, FuncArgumentKind},
        authoring::FuncAuthoringClient,
        binding::{
            action::ActionBinding, attribute::AttributeBinding, authentication::AuthBinding,
            leaf::LeafBinding, AttributeArgumentBinding, AttributeFuncArgumentSource,
            AttributeFuncDestination, EventualParent, FuncBinding, FuncBindingError,
        },
    },
    prop::PropPath,
    schema::variant::{
        authoring::VariantAuthoringClient,
        leaves::{LeafInputLocation, LeafKind},
    },
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
    AttributePrototype, DalContext, Func, Prop, Schema, SchemaVariant, SchemaVariantError,
    WorkspaceSnapshotError,
};
use dal_test::{helpers::create_unlocked_variant_copy_for_schema_name, test};
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
    dbg!(&bindings);
    assert_eq!(bindings.len(), 1);

    let binding = bindings.pop().expect("has a binding");

    let old_schema_variant_id = binding.get_schema_variant().expect("has a schema variant");

    // this schema variant is locked
    let old_schema_variant = SchemaVariant::get_by_id_or_error(ctx, old_schema_variant_id)
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
        let sv =
            SchemaVariant::get_by_id_or_error(ctx, binding.get_schema_variant().expect("has sv"))
                .await
                .expect("has sv");
        dbg!(sv);
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

    dbg!(&latest_sv);
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
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

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
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

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
    let schema = Schema::find_by_name(ctx, "katy perry")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

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
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

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

    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

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

    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

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
async fn code_gen_cannot_create_cycle(ctx: &mut DalContext) {
    let _schema = Schema::find_by_name(ctx, "katy perry")
        .await
        .expect("could not perform find by name")
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
        Err(FuncBindingError::SchemaVariant(SchemaVariantError::AttributePrototypeArgument(
            AttributePrototypeArgumentError::WorkspaceSnapshot(
                WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                    WorkspaceSnapshotGraphError::CreateGraphCycle,
                ),
            ),
        ))) => {}
        _ => panic!("Test should fail if we don't get this error"),
    }
}
