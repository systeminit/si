use dal::action::prototype::ActionKind;
use dal::func::authoring::FuncAuthoringClient;
use dal::func::summary::FuncSummary;
use dal::func::view::FuncView;
use dal::func::{FuncAssociations, FuncKind};
use dal::{DalContext, Func, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn attach_multiple_action_funcs(ctx: &mut DalContext) {
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

    // Attach one action func to the schema variant and commit.
    let func_id = Func::find_by_name(ctx, "test:createActionFallout")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let (_, mut schema_variant_ids) = func_view
        .associations
        .expect("empty associations")
        .get_action_internals()
        .expect("could not get internals");
    schema_variant_ids.push(schema_variant_id);
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Action {
            kind: ActionKind::Create,
            schema_variant_ids,
        }),
    )
    .await
    .expect("unable to save the func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Attach a second action func to the same schema variant and commit.
    let func_id = Func::find_by_name(ctx, "test:deleteActionSwifty")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let (_, mut schema_variant_ids) = func_view
        .associations
        .expect("empty associations")
        .get_action_internals()
        .expect("could not get internals");
    schema_variant_ids.push(schema_variant_id);
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Action {
            kind: ActionKind::Destroy,
            schema_variant_ids,
        }),
    )
    .await
    .expect("unable to save the func");
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

#[test]
async fn error_when_attaching_an_exisiting_type(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("unable to find by name")
        .expect("no schema found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");
    let func_id = Func::find_by_name(ctx, "test:createActionFallout")
        .await
        .expect("unable to find the func");
    assert!(func_id.is_some());

    let new_action_func_name = "anotherCreate";
    FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Action,
        Some(new_action_func_name.to_string()),
        None,
    )
    .await
    .expect("could not create func");

    let func_id = Func::find_by_name(ctx, new_action_func_name)
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let (func_view_kind, mut schema_variant_ids) = func_view
        .associations
        .expect("empty associations")
        .get_action_internals()
        .expect("could not get internals");
    schema_variant_ids.push(schema_variant_id);
    assert!(FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Action {
            kind: func_view_kind,
            schema_variant_ids,
        }),
    )
    .await
    .is_err());
}

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
    let func_id = Func::find_by_name(ctx, "test:setDummySecretString")
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let mut schema_variant_ids = func_view
        .associations
        .expect("empty associations")
        .get_authentication_internals()
        .expect("could not get internals");
    schema_variant_ids.push(schema_variant_id);
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Authentication { schema_variant_ids }),
    )
    .await
    .expect("unable to save the func");
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
    let func_id = Func::find_by_name(ctx, new_auth_func_name)
        .await
        .expect("unable to find the func")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("unable to get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("unable to assemble a func view");
    let mut schema_variant_ids = func_view
        .associations
        .expect("empty associations")
        .get_authentication_internals()
        .expect("could not get internals");
    schema_variant_ids.push(schema_variant_id);
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::Authentication { schema_variant_ids }),
    )
    .await
    .expect("unable to save the func");
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
