use dal::schema::variant::root_prop::RootPropChild;
use dal::{
    schema::{variant::leaves::LeafKind, SchemaVariant},
    DalContext, Prop,
};
use dal_test::{test, test_harness::create_schema};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let schema = create_schema(ctx).await;

    let (variant, _) = SchemaVariant::new(ctx, schema.id(), "ringo", "beatles")
        .await
        .expect("cannot create schema variant");
    assert_eq!(variant.name(), "ringo");
}

#[test]
async fn find_code_item_prop(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = SchemaVariant::new(ctx, schema.id(), "v0", "v0")
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
    let schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = SchemaVariant::new(ctx, schema.id(), "v0", "v0")
        .await
        .expect("cannot create schema variant");

    // Gather all children of "/root/si".
    let expected_si_child_prop_ids = Prop::direct_child_prop_ids(ctx, root_prop.si_prop_id)
        .await
        .expect("could not get direct child prop ids");

    // Now, test our query.
    let found_si_prop =
        SchemaVariant::find_root_child_prop_id(ctx, schema_variant.id(), RootPropChild::Si)
            .await
            .expect("could not get si prop");
    let found_si_child_prop_ids = Prop::direct_child_prop_ids(ctx, found_si_prop)
        .await
        .expect("could not get direct child prop ids");

    assert_eq!(
        expected_si_child_prop_ids, // expected
        found_si_child_prop_ids,    // actual
    )
}
