use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, DalContext, Schema, SchemaVariant};
use dal_test::{
    expected::{
        ExpectSchema, apply_change_set_to_base, fork_from_head_change_set,
        update_visibility_and_snapshot_to_visibility,
    },
    helpers::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;

/// We don't have a good way of simulating two concurrent requests coming in to SDF at the
/// same time to make a change to a Change Set, and trying to simulate it using two Change Sets as the
/// "requests", and the base Change Set as the Change Set those requests are attempting to modify
/// doesn't replicate the scenario either.
#[ignore]
#[test]
async fn keeps_only_one_unlocked_variant_from_concurrent_requests(ctx: &mut DalContext) {
    let expect_schema = ExpectSchema::find(ctx, "Docker Image").await;
    let default_expect_variant = expect_schema.default_variant(ctx).await;
    let _default_schema_variant = default_expect_variant.schema_variant(ctx).await;

    let first_request_change_set =
        ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, "First request Change Set")
            .await
            .expect("Unable to create new Change Set");

    let first_unlocked_variant = default_expect_variant.create_unlocked_copy(ctx).await;

    let mut schema_variants = SchemaVariant::list_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to list SchemaVariants");
    schema_variants.retain(|sv| !sv.is_locked());
    assert_eq!(1, schema_variants.len());

    let unlocked_schema_variant = SchemaVariant::get_unlocked_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to get unlocked SchemaVariant")
        .expect("No unlocked SchemaVariant found");

    assert_eq!(first_unlocked_variant.id(), unlocked_schema_variant.id());

    let second_request_change_set =
        ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, "Second request Change Set")
            .await
            .expect("Unable to create new Change Set");

    let second_unlocked_variant = default_expect_variant.create_unlocked_copy(ctx).await;

    let mut schema_variants = SchemaVariant::list_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to list SchemaVariants");
    schema_variants.retain(|sv| !sv.is_locked());
    assert_eq!(1, schema_variants.len());

    let unlocked_schema_variant = SchemaVariant::get_unlocked_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to get unlocked SchemaVariant")
        .expect("No unlocked SchemaVariant found");

    assert_eq!(second_unlocked_variant.id(), unlocked_schema_variant.id());

    ctx.update_visibility_and_snapshot_to_visibility(first_request_change_set.id)
        .await
        .expect("Unable to update ctx to first request change set");
    // NOTE: We *CANNOT* use `ChangeSetTestHelpers::apply_change_set_to_base` as it explicitly
    // locks all editing SchemaVariants, which defeats what we're attempting to test.
    ChangeSet::apply_to_base_change_set(ctx)
        .await
        .expect("Unable to apply first request to base change set");

    ctx.update_visibility_and_snapshot_to_visibility(
        first_request_change_set
            .base_change_set_id
            .expect("Orphaned Change Set"),
    )
    .await
    .expect("Unable to update ctx to base Change Set");

    let mut schema_variants = SchemaVariant::list_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to list SchemaVariants");
    schema_variants.retain(|sv| !sv.is_locked());
    assert_eq!(1, schema_variants.len());

    let unlocked_schema_variant = SchemaVariant::get_unlocked_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to get unlocked SchemaVariant")
        .expect("No unlocked SchemaVariant found");

    assert_eq!(first_unlocked_variant.id(), unlocked_schema_variant.id());

    ctx.update_visibility_and_snapshot_to_visibility(second_request_change_set.id)
        .await
        .expect("Unable to update ctx to second request change set");
    ChangeSet::apply_to_base_change_set(ctx)
        .await
        .expect("Unable to apply second request to base change set");

    ctx.update_visibility_and_snapshot_to_visibility(
        second_request_change_set
            .base_change_set_id
            .expect("Orphaned Change Set"),
    )
    .await
    .expect("Unable to update ctx to base Change Set");

    let mut schema_variants = SchemaVariant::list_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to list SchemaVariants");
    schema_variants.retain(|sv| !sv.is_locked());
    assert_eq!(1, schema_variants.len());

    let unlocked_schema_variant = SchemaVariant::get_unlocked_for_schema(ctx, expect_schema.id())
        .await
        .expect("Unable to get unlocked SchemaVariant")
        .expect("No unlocked SchemaVariant found");

    assert_eq!(second_unlocked_variant.id(), unlocked_schema_variant.id());
}

#[ignore]
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

    let sv_cs_1 = SchemaVariant::get_by_id(ctx, updated_sv_id_cs_1)
        .await
        .expect("unable to get the updated sv");
    sv_cs_1
        .lock(ctx)
        .await
        .expect("unable to lock the schema variant");
    schema
        .set_default_variant_id(ctx, updated_sv_id_cs_1)
        .await
        .expect("unable to update the default schema variant id");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    assert!(
        SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_1)
            .await
            .expect("get defaultness for sv from cs_1")
    );

    // Fork and create a new variant as the default in another change set, then
    // apply that to head
    let _cs_2 = fork_from_head_change_set(ctx).await;

    let updated_sv_id_cs_2 =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, variant.id())
            .await
            .expect("unable to create variant copy")
            .id();

    let sv_cs_2 = SchemaVariant::get_by_id(ctx, updated_sv_id_cs_2)
        .await
        .expect("unable to get the updated sv");
    sv_cs_2
        .lock(ctx)
        .await
        .expect("unable to lock the schema variant");
    schema
        .set_default_variant_id(ctx, updated_sv_id_cs_2)
        .await
        .expect("unable to update the default schema variant id");
    assert!(
        SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_2)
            .await
            .expect("get defaultness for sv from cs_1")
    );

    apply_change_set_to_base(ctx).await;

    assert!(
        SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_2)
            .await
            .expect("get defaultness for sv from cs_2")
    );

    assert_eq!(
        2,
        Schema::list_schema_variant_ids(ctx, schema.id())
            .await
            .expect("able to list svs")
            .len()
    );

    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

    assert!(
        SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_2)
            .await
            .expect("get defaultness for sv from cs_2")
    );

    assert!(
        !SchemaVariant::is_default_by_id(ctx, updated_sv_id_cs_1)
            .await
            .expect("get defaultness for sv from cs_1")
    );

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
