use dal::func::authoring::FuncAuthoringClient;
use dal::func::binding::authentication::AuthBinding;
use dal::schema::variant::authoring::VariantAuthoringClient;
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
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    let total_funcs = funcs.len();

    // create unlocked copy
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("could create unlocked copy")
            .id();
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
    FuncAuthoringClient::create_new_auth_func(
        ctx,
        Some(new_auth_func_name.to_string()),
        schema_variant_id,
    )
    .await
    .expect("could not create auth func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see the two that were attached.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    assert_eq!(
        total_funcs + 2, // expected
        funcs.len()      // actual
    );
}

#[test]
async fn detach_auth_func(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");

    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    let total_funcs = funcs.len();

    // Get the Auth Func
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found auth func")
        .expect("has a func");

    // Try to detach and see it fails
    let delete_result = AuthBinding::delete_auth_binding(ctx, func_id, schema_variant_id).await;
    assert!(delete_result.is_err());

    // now create unlocked copy
    let unlocked_schema_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("could create unlocked copy")
            .id();

    // detach auth func for unlocked copy
    AuthBinding::delete_auth_binding(ctx, func_id, unlocked_schema_variant)
        .await
        .expect("could not delete auth binding");

    // check that there's one less func
    let funcs = SchemaVariant::all_funcs(ctx, unlocked_schema_variant)
        .await
        .expect("could not list funcs for schema variant");
    assert_eq!(funcs.len(), total_funcs - 1);
}

#[test]
async fn edit_auth_func(ctx: &mut DalContext) {
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
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");

    // Get the Auth Func
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found auth func")
        .expect("has a func");

    // ensure the func is attached
    assert!(funcs.into_iter().any(|func| func.id == func_id));

    // create unlocked copy of it
    let unlocked_func_id = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could not create unlocked copy");

    // find unlocked copy of the variant
    let unlocked_schema_variant = SchemaVariant::get_unlocked_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant")
        .expect("has unlocked variant");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    assert!(!unlocked_schema_variant.is_locked());

    // ensure the unlocked variant has the new func attached and not the old one
    let funcs = SchemaVariant::all_funcs(ctx, unlocked_schema_variant.id)
        .await
        .expect("could not list funcs for schema variant");

    assert!(funcs
        .clone()
        .into_iter()
        .any(|func| func.id == unlocked_func_id.id));
    assert!(funcs.into_iter().any(|func| func.id != func_id));

    // edit the func
    let new_auth_func_code = "async function auth(secret: Input): Promise<Output> { requestStorage.setItem('dummySecretString', secret.value); requestStorage.setItem('workspaceToken', secret.WorkspaceToken); console.log('success');}";

    FuncAuthoringClient::save_code(ctx, unlocked_func_id.id, new_auth_func_code.to_string())
        .await
        .expect("could not save code");

    let unlocked_func = Func::get_by_id_or_error(ctx, unlocked_func_id.id)
        .await
        .expect("could not get func");
    // ensure it saved
    let maybe_new_code = unlocked_func
        .code_plaintext()
        .expect("got code")
        .expect("has code");

    assert_eq!(new_auth_func_code.to_string(), maybe_new_code);

    // commit and apply
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply to base");

    // ensure new func is locked
    let new_locked_func = Func::get_by_id_or_error(ctx, unlocked_func.id)
        .await
        .expect("could not get func");

    assert!(new_locked_func.is_locked);
    assert_eq!(
        new_locked_func
            .code_plaintext()
            .expect("got code")
            .expect("has code"),
        new_auth_func_code.to_string()
    );

    //ensure new schema variant is locked and default
    let maybe_locked_schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");
    let maybe_locked_schema_variant =
        SchemaVariant::get_by_id_or_error(ctx, maybe_locked_schema_variant_id)
            .await
            .expect("could not get schema variant");
    assert!(maybe_locked_schema_variant.is_locked());
}
