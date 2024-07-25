use dal::func::summary::FuncSummary;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{DalContext, Func, Schema, SchemaVariant};

use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn regenerate_variant(ctx: &mut DalContext) {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");
    // find the variant we know is default and attached to this func already
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("unable to find by name")
        .expect("no schema found");

    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");
    // Cache the total number of funcs before continuing.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");

    // Get the Auth Func
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found auth func")
        .expect("has a func");

    // ensure the func is attached
    assert!(funcs.into_iter().any(|func| func.id == func_id));

    // unlock schema variant
    let unlocked_schema_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("could not unlock variant");

    // ensure func is attached to new variant
    let funcs_for_unlocked =
        FuncSummary::list_for_schema_variant_id(ctx, unlocked_schema_variant.id)
            .await
            .expect("unable to get the funcs for a schema variant");

    // ensure the func is attached
    assert!(funcs_for_unlocked
        .into_iter()
        .any(|func| func.id == func_id));

    // get the existing default variant and ensure the auth func is still attached to it
    let funcs_for_default = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    // ensure the func is attached
    assert!(funcs_for_default.into_iter().any(|func| func.id == func_id));

    // regenerate variant
    VariantAuthoringClient::regenerate_variant(ctx, unlocked_schema_variant.id)
        .await
        .expect("could not regenerate variant");

    // ensure funcs are attached to regenerated AND the existing default
    // ensure func is attached to new variant
    let funcs_for_unlocked =
        FuncSummary::list_for_schema_variant_id(ctx, unlocked_schema_variant.id)
            .await
            .expect("unable to get the funcs for a schema variant");

    // ensure the func is attached
    assert!(funcs_for_unlocked
        .into_iter()
        .any(|func| func.id == func_id));

    // get the existing default variant and ensure the auth func is still attached to it
    let funcs_for_default = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    // ensure the func is attached
    assert!(funcs_for_default.into_iter().any(|func| func.id == func_id));
}
