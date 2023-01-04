use dal::{
    schema::{variant::leaves::LeafKind, SchemaVariant},
    DalContext, InternalProvider, StandardModel,
};
use dal_test::{test, test_harness::create_schema};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let schema = create_schema(ctx).await;

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "ringo")
        .await
        .expect("cannot create schema variant");
    assert_eq!(variant.name(), "ringo");
}

#[test]
async fn set_schema(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");

    let attached_schema = variant
        .schema(ctx)
        .await
        .expect("cannot get schema")
        .expect("should have a schema");
    assert_eq!(schema, attached_schema);

    variant
        .unset_schema(ctx)
        .await
        .expect("cannot unassociate variant with schema");
    let attached_schema = variant.schema(ctx).await.expect("cannot get schema");
    assert_eq!(attached_schema, None);
}

#[test]
async fn find_code_item_prop(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");

    // Check that our query works to find "/root/code/codeItem".
    let found_code_item_prop =
        SchemaVariant::find_leaf_item_prop(ctx, *schema_variant.id(), LeafKind::CodeGeneration)
            .await
            .expect("could not find code item prop");
    assert_eq!("codeItem", found_code_item_prop.name());

    // Check that the parent is "/root/code".
    let found_code_map_prop = found_code_item_prop
        .parent_prop(ctx)
        .await
        .expect("could not perform find parent prop")
        .expect("parent prop not found");
    assert_eq!(root_prop.code_prop_id, *found_code_map_prop.id());
}

#[test]
async fn find_implicit_internal_providers_for_root_children(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");
    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize schema variant");

    let found_domain_internal_provider =
        SchemaVariant::find_domain_implicit_internal_provider(ctx, *schema_variant.id())
            .await
            .expect("could not find internal provider");

    let expected_domain_internal_provider =
        InternalProvider::find_for_prop(ctx, root_prop.domain_prop_id)
            .await
            .expect("could not perform find for prop")
            .expect("internal provider not found");

    assert_eq!(
        *expected_domain_internal_provider.id(),
        *found_domain_internal_provider.id()
    );

    let found_code_internal_provider =
        SchemaVariant::find_code_implicit_internal_provider(ctx, *schema_variant.id())
            .await
            .expect("could not find internal provider");

    let expected_code_internal_provider =
        InternalProvider::find_for_prop(ctx, root_prop.code_prop_id)
            .await
            .expect("could not perform find for prop")
            .expect("internal provider not found");

    assert_eq!(
        *expected_code_internal_provider.id(),
        *found_code_internal_provider.id()
    );
}
