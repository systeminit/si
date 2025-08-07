use dal::{
    ChangeSet,
    DalContext,
    Schema,
    SchemaVariant,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::test;

#[test]
async fn clone_variant(ctx: &mut DalContext) {
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let schema = Schema::get_by_name(ctx, "dummy-secret")
        .await
        .expect("schema not found");

    let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("Unable to find the default schema variant id");
    let existing_variant = SchemaVariant::get_by_id(ctx, default_schema_variant)
        .await
        .expect("unable to lookup the default schema variant");

    let clone_name = format!("{}-Clone", schema.name());
    let (new_schema_variant, _) = VariantAuthoringClient::new_schema_with_cloned_variant(
        ctx,
        default_schema_variant,
        clone_name,
    )
    .await
    .expect("unable to clone the schema variant");

    assert_eq!(new_schema_variant.category(), existing_variant.category());
    assert_eq!(
        new_schema_variant.display_name(),
        format!("{}-Clone", existing_variant.display_name())
    );
    assert_eq!(
        new_schema_variant
            .get_color(ctx)
            .await
            .expect("unable to get color"),
        new_schema_variant
            .get_color(ctx)
            .await
            .expect("unable to get color")
    );
    assert!(new_schema_variant.asset_func_id().is_some());

    assert_ne!(new_schema_variant.id(), default_schema_variant);
}
