use dal::{
    ComponentType,
    DalContext,
    Func,
    Prop,
    Schema,
    func::leaf::LeafKind,
    schema::{
        SchemaVariant,
        variant::root_prop::RootPropChild,
    },
};
use dal_test::{
    helpers::create_schema,
    test,
};
use pretty_assertions_sorted::assert_eq;

mod view;

#[test]
async fn new(ctx: &DalContext) {
    let schema = create_schema(ctx).await.expect("could not create schema");

    let (variant, _) = SchemaVariant::new(
        ctx,
        schema.id(),
        "ringo starr",
        "ringo".to_string(),
        "beatles",
        "#FFFFFF",
        ComponentType::Component,
        None,
        None,
        None,
        false,
    )
    .await
    .expect("cannot create schema variant");
    assert_eq!(variant.version(), "ringo starr");
}

#[test]
async fn find_code_item_prop(ctx: &DalContext) {
    let schema = create_schema(ctx).await.expect("could not create schema");
    let (schema_variant, root_prop) = SchemaVariant::new(
        ctx,
        schema.id(),
        "v0",
        "v0_display_name".to_string(),
        "demo",
        "#000000",
        ComponentType::Component,
        None,
        None,
        None,
        false,
    )
    .await
    .expect("cannot create schema variant");

    // Check that our query works to find "/root/code/codeItem".
    let found_code_item_prop_id =
        SchemaVariant::find_leaf_item_prop(ctx, schema_variant.id(), LeafKind::CodeGeneration)
            .await
            .expect("could not find code item prop");

    // Check that the parent is "/root/code".
    let found_code_map_prop_id = Prop::parent_prop_id_by_id(ctx, found_code_item_prop_id)
        .await
        .expect("could not perform find parent prop")
        .expect("parent prop not found");
    assert_eq!(root_prop.code_prop_id, found_code_map_prop_id);
}

#[test]
async fn list_root_si_child_props(ctx: &DalContext) {
    let schema = create_schema(ctx).await.expect("could not create schema");
    let (schema_variant, root_prop) = SchemaVariant::new(
        ctx,
        schema.id(),
        "v0",
        "v0_display_name".to_string(),
        "demo",
        "#000000",
        ComponentType::Component,
        None,
        None,
        None,
        false,
    )
    .await
    .expect("cannot create schema variant");

    // Gather all children of "/root/si".
    let expected_si_child_prop_ids =
        Prop::direct_child_prop_ids_unordered(ctx, root_prop.si_prop_id)
            .await
            .expect("could not get direct child prop ids");

    // Now, test our query.
    let found_si_prop =
        SchemaVariant::find_root_child_prop_id(ctx, schema_variant.id(), RootPropChild::Si)
            .await
            .expect("could not get si prop");
    let found_si_child_prop_ids = Prop::direct_child_prop_ids_unordered(ctx, found_si_prop)
        .await
        .expect("could not get direct child prop ids");

    assert_eq!(
        expected_si_child_prop_ids, // expected
        found_si_child_prop_ids,    // actual
    )
}

#[test]
async fn all_prop_ids(ctx: &DalContext) {
    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get schema variant");

    let all_prop_ids = SchemaVariant::all_prop_ids(ctx, schema_variant_id)
        .await
        .expect("could not list all prop ids");
    let mut prop_paths = Vec::new();
    for prop_id in all_prop_ids {
        let prop_path = Prop::path_by_id(ctx, prop_id)
            .await
            .expect("could not get path");
        prop_paths.push(prop_path.with_replaced_sep("/").to_string());
    }
    prop_paths.sort();

    // NOTE(nick): this is going to be annoying to maintain if we are frequently changing the props
    // on the schema variant. We want this test to make sure _every_ prop comes back, as expected,
    // so maybe there is another way?
    let expected = [
        "root",
        "root/code",
        "root/code/codeItem",
        "root/code/codeItem/code",
        "root/code/codeItem/format",
        "root/deleted_at",
        "root/domain",
        "root/domain/attributes",
        "root/domain/freestar",
        "root/domain/hidden_prop",
        "root/domain/name",
        "root/domain/possible_world_a",
        "root/domain/possible_world_a/wormhole_1",
        "root/domain/possible_world_a/wormhole_1/wormhole_2",
        "root/domain/possible_world_a/wormhole_1/wormhole_2/wormhole_3",
        "root/domain/possible_world_a/wormhole_1/wormhole_2/wormhole_3/rigid_designator",
        "root/domain/possible_world_b",
        "root/domain/possible_world_b/wormhole_1",
        "root/domain/possible_world_b/wormhole_1/wormhole_2",
        "root/domain/possible_world_b/wormhole_1/wormhole_2/wormhole_3",
        "root/domain/possible_world_b/wormhole_1/wormhole_2/wormhole_3/naming_and_necessity",
        "root/domain/universe",
        "root/domain/universe/galaxies",
        "root/domain/universe/galaxies/galaxy",
        "root/domain/universe/galaxies/galaxy/planets",
        "root/domain/universe/galaxies/galaxy/sun",
        "root/qualification",
        "root/qualification/qualificationItem",
        "root/qualification/qualificationItem/message",
        "root/qualification/qualificationItem/result",
        "root/resource",
        "root/resource/last_synced",
        "root/resource/message",
        "root/resource/payload",
        "root/resource/status",
        "root/resource_value",
        "root/secrets",
        "root/si",
        "root/si/color",
        "root/si/name",
        "root/si/protected",
        "root/si/resourceId",
        "root/si/tags",
        "root/si/tags/tag",
        "root/si/type",
    ];
    assert_eq!(
        expected
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>(), // expected
        prop_paths // actual
    );
}

#[test]
async fn all_funcs(ctx: &DalContext) {
    let schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get schema variant");
    let all_funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("unable to get all funcs");

    let (expected, actual) = prepare_for_assertion(
        &[
            "si:identity",
            "si:resourcePayloadToValue",
            "si:unset",
            "test:createActionSwifty",
            "test:deleteActionSwifty",
            "test:generateCode",
            "test:refreshActionSwifty",
            "test:swiftyQualification",
            "test:updateActionSwifty",
        ],
        all_funcs.as_slice(),
    );
    assert_eq!(expected, actual);

    let schema = Schema::get_by_name(ctx, "starfield")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get schema variant");
    let all_funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("unable to get all funcs");

    let (expected, actual) = prepare_for_assertion(
        &[
            "hesperus_is_phosphorus",
            "si:identity",
            "si:resourcePayloadToValue",
            "si:unset",
            "test:createActionStarfield",
            "test:falloutEntriesToGalaxies",
            "test:refreshActionStarfield",
        ],
        all_funcs.as_slice(),
    );
    assert_eq!(expected, actual);
}

#[test]
async fn list_user_facing_works(ctx: &DalContext) {
    SchemaVariant::list_user_facing(ctx)
        .await
        .expect("could not list user facing schema variants");
}

#[test]
async fn find_schema_case_insensitive(ctx: &DalContext) {
    let schema_name = "AWS::VpcLattice::Service";
    let schema = Schema::new(ctx, schema_name)
        .await
        .expect("could not create schema");

    assert_eq!(schema_name, schema.name());

    let found_schema = Schema::get_by_name_opt(ctx, "aws::vpclattice::service")
        .await
        .expect("could not search for schema")
        .expect("schema not found with lowercase name");

    assert_eq!(schema.id(), found_schema.id());
    assert_eq!(schema.name(), found_schema.name());

    let found_schema_upper = Schema::get_by_name_opt(ctx, "AWS::VPCLattice::Service")
        .await
        .expect("could not search for schema")
        .expect("schema not found with mixed case name");

    assert_eq!(schema.id(), found_schema_upper.id());
    assert_eq!(schema.name(), found_schema_upper.name());

    let found_schema_mixed = Schema::get_by_name_opt(ctx, "AWS::VPCLATTICE::SERVICE")
        .await
        .expect("could not search for schema")
        .expect("schema not found with upper case name");

    assert_eq!(schema.id(), found_schema_mixed.id());
    assert_eq!(schema.name(), found_schema_mixed.name());
}

fn prepare_for_assertion(expected: &[&str], all_funcs: &[Func]) -> (Vec<String>, Vec<String>) {
    let expected = expected.iter().map(|s| s.to_string()).collect();

    let mut actual: Vec<String> = all_funcs.iter().map(|f| f.name.clone()).collect();
    actual.sort();

    (expected, actual)
}
