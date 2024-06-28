use dal::func::authoring::FuncAuthoringClient;
use dal::func::binding::authentication::AuthBinding;
use dal::func::summary::FuncSummary;
use dal::func::FuncKind;
use dal::{DalContext, Func, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn attach_multiple_auth_funcs_with_creation(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "katy perry")
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
    let total_funcs = funcs.len();

    // Attach one auth func to the schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, "test:setDummySecretString")
        .await
        .expect("unable to find the func")
        .expect("no func found");

    AuthBinding::create_auth_binding(ctx, func_id, schema_variant_id)
        .await
        .expect("could not create auth binding");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create an auth func to be attached and commit.
    let new_auth_func_name = "shattered space";
    FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Authentication,
        Some(new_auth_func_name.to_string()),
        None,
    )
    .await
    .expect("could not create func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Attach a second auth func (the new one) to the same schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, new_auth_func_name)
        .await
        .expect("unable to find the func")
        .expect("no func found");
    AuthBinding::create_auth_binding(ctx, func_id, schema_variant_id)
        .await
        .expect("could not create auth binding");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see the two that were attached.
    let funcs = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("unable to get the funcs for a schema variant");
    assert_eq!(
        total_funcs + 2, // expected
        funcs.len()      // actual
    );
}
