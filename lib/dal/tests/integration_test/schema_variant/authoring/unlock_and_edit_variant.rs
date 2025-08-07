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
async fn create_variant_merge_unlock_and_edit(ctx: &mut DalContext) {
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

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    let schema = variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let asset_func = Func::get_by_id(
        ctx,
        variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get asset authoring func");

    assert!(!asset_func.is_locked);

    // Now let's update the variant
    let new_code = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()\n  return new AssetBuilder().addProp(myProp).build()\n}".to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        variant.id(),
        &schema.name,
        variant.display_name(),
        variant.category(),
        variant.description(),
        variant.link(),
        variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        variant.component_type(),
        Some(new_code),
    )
    .await
    .expect("save variant contents");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    let updated_sv_id = VariantAuthoringClient::regenerate_variant(ctx, variant.id())
        .await
        .expect("unable to update asset");

    let sv = SchemaVariant::get_by_id(ctx, updated_sv_id)
        .await
        .expect("unable to get the updated sv");
    sv.lock(ctx)
        .await
        .expect("unable to lock the schema variant");
    schema
        .set_default_variant_id(ctx, updated_sv_id)
        .await
        .expect("unable to update the default schema variant id");
    asset_func
        .lock(ctx)
        .await
        .expect("unable to lock the asset func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let sv = SchemaVariant::get_by_id(ctx, updated_sv_id)
        .await
        .expect("unable to get the new schema variant");

    assert!(sv.is_locked());

    let asset_func = Func::get_by_id(
        ctx,
        variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get the updated asset func");

    assert!(asset_func.is_locked);

    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("unable to create a new changeset");

    let unlocked_schema_variant = VariantAuthoringClient::create_unlocked_variant_copy(ctx, sv.id)
        .await
        .expect("unable to create an unlocked copy of a schema variant");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    let unlocked_asset_func = Func::get_by_id(
        ctx,
        unlocked_schema_variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get the updated asset func");

    assert!(!unlocked_asset_func.is_locked);

    // Now let's update the variant
    let new_code = "function main() {return new AssetBuilder().build()\n}".to_string();
    let res = VariantAuthoringClient::save_variant_content(
        ctx,
        unlocked_schema_variant.id(),
        &schema.name,
        unlocked_schema_variant.display_name(),
        unlocked_schema_variant.category(),
        unlocked_schema_variant.description(),
        unlocked_schema_variant.link(),
        unlocked_schema_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        unlocked_schema_variant.component_type(),
        Some(new_code),
    )
    .await;

    assert!(res.is_ok());
}
