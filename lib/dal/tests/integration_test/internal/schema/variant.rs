use dal::{
    schema::{variant::leaves::LeafKind, SchemaVariant},
    DalContext, InternalProvider, Prop, PropId, RootPropChild, StandardModel,
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
async fn list_root_si_child_props(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    // Gather all children of "/root/si".
    let si_prop = Prop::get_by_id(ctx, &root_prop.si_prop_id)
        .await
        .expect("could not perform get by id")
        .expect("prop not found");
    let expected_si_child_props = si_prop
        .child_props(ctx)
        .await
        .expect("could not find child props");
    let expected_si_child_prop_ids: Vec<PropId> =
        expected_si_child_props.iter().map(|p| *p.id()).collect();

    // Now, test our query.
    let found_si_child_props = SchemaVariant::list_root_si_child_props(ctx, *schema_variant.id())
        .await
        .expect("could not list root si child props");
    let found_si_child_prop_ids: Vec<PropId> =
        found_si_child_props.iter().map(|p| *p.id()).collect();

    assert_eq!(
        expected_si_child_prop_ids, // expected
        found_si_child_prop_ids,    // actual
    )
}

#[test]
async fn list_implicit_internal_providers_for_root_children(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    let children = [
        (RootPropChild::Si, root_prop.si_prop_id),
        (RootPropChild::Domain, root_prop.domain_prop_id),
        (RootPropChild::Resource, root_prop.resource_prop_id),
        (RootPropChild::Code, root_prop.code_prop_id),
        (
            RootPropChild::Qualification,
            root_prop.qualification_prop_id,
        ),
        (RootPropChild::Confirmation, root_prop.confirmation_prop_id),
    ];

    for (child, prop_id) in children {
        let found_implicit_internal_provider =
            SchemaVariant::find_root_child_implicit_internal_provider(
                ctx,
                *schema_variant.id(),
                child,
            )
            .await
            .expect("could not find internal provider");
        let expected_implicit_internal_provider = InternalProvider::find_for_prop(ctx, prop_id)
            .await
            .expect("could not perform find for prop")
            .expect("internal provider not found");
        assert_eq!(
            *expected_implicit_internal_provider.id(),
            *found_implicit_internal_provider.id()
        );
    }
}
