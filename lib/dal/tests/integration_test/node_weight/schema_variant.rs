use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{DalContext, Schema, SchemaVariant};
use dal_test::expected::{
    apply_change_set_to_base, fork_from_head_change_set,
    update_visibility_and_snapshot_to_visibility,
};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn only_one_default_schema_variant(ctx: &mut DalContext) {
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

    apply_change_set_to_base(ctx).await;

    let schema = variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    // Fork and create a new variant as the default
    let cs_1 = fork_from_head_change_set(ctx).await;

    let updated_sv_id_cs_1 =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, variant.id())
            .await
            .expect("unable to create variant copy")
            .id();

    let sv_cs_1 = SchemaVariant::get_by_id_or_error(ctx, updated_sv_id_cs_1)
        .await
        .expect("unable to get the updated sv");
    sv_cs_1
        .lock(ctx)
        .await
        .expect("unable to lock the schema variant");
    schema
        .set_default_schema_variant(ctx, updated_sv_id_cs_1)
        .await
        .expect("unable to update the default schema variant id");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    assert!(SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_1)
        .await
        .expect("get defaultness for sv from cs_1"));

    // Fork and create a new variant as the default in another change set, then
    // apply that to head
    let _cs_2 = fork_from_head_change_set(ctx).await;

    let updated_sv_id_cs_2 =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, variant.id())
            .await
            .expect("unable to create variant copy")
            .id();

    let sv_cs_2 = SchemaVariant::get_by_id_or_error(ctx, updated_sv_id_cs_2)
        .await
        .expect("unable to get the updated sv");
    sv_cs_2
        .lock(ctx)
        .await
        .expect("unable to lock the schema variant");
    schema
        .set_default_schema_variant(ctx, updated_sv_id_cs_2)
        .await
        .expect("unable to update the default schema variant id");
    assert!(SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_2)
        .await
        .expect("get defaultness for sv from cs_1"));

    apply_change_set_to_base(ctx).await;

    assert!(SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_2)
        .await
        .expect("get defaultness for sv from cs_2"));

    assert_eq!(
        2,
        Schema::list_schema_variant_ids(ctx, schema.id())
            .await
            .expect("able to list svs")
            .len()
    );

    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

    assert!(SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_2)
        .await
        .expect("get defaultness for sv from cs_2"));

    assert!(!SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_1)
        .await
        .expect("get defaultness for sv from cs_1"));

    // should be 3 now, the original, the one made in cs_2 and the one made in
    // cs_1. This ensures we added the use edges back for the previous defaults
    assert_eq!(
        3,
        Schema::list_schema_variant_ids(ctx, schema.id())
            .await
            .expect("able to list svs")
            .len()
    );
}
