use dal::{
    DalContext,
    Func,
    SchemaVariant,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    helpers::ChangeSetTestHelpers,
    test,
};

#[test]
async fn delete_unlocked_variant(ctx: &mut DalContext) {
    let asset_name = "chainsawVariant".to_string();
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await
    .expect("Unable to create new asset");

    let variant_id = variant.id();

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    let asset_func = Func::get_by_id(
        ctx,
        variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get asset authoring func");

    assert!(!asset_func.is_locked);

    let locked_variant = variant
        .lock(ctx)
        .await
        .expect("unable to lock the schema variant");
    asset_func
        .lock(ctx)
        .await
        .expect("unable to lock the asset func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    // We can't delete the variant in a locked format
    let res = SchemaVariant::cleanup_unlocked_variant(ctx, locked_variant.id).await;
    assert!(res.is_err());

    // let's create an unlocked copy to ensure we can remove it
    let unlocked_schema_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, variant_id)
            .await
            .expect("unable to create an unlocked copy of a schema variant");

    let res = SchemaVariant::cleanup_unlocked_variant(ctx, unlocked_schema_variant.id).await;
    assert!(res.is_ok());
}
