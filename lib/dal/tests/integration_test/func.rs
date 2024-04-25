use dal::func::authoring::FuncAuthoringClient;
use dal::func::summary::FuncSummary;
use dal::func::FuncKind;
use dal::{DalContext, Func, Schema, SchemaVariant};
use dal_test::helpers::create_empty_action_func;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

mod argument;
mod associations;
mod authoring;

#[test]
async fn summary(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema")
        .expect("schema not found");
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("no schema variant found");

    // Ensure that the same func can be found within its schema variant and for all funcs in the workspace.
    let funcs_for_schema_variant = FuncSummary::list_for_schema_variant_id(ctx, schema_variant_id)
        .await
        .expect("could not list func summaries");
    let all_funcs = FuncSummary::list(ctx)
        .await
        .expect("could not list func summaries");

    let func_name = "test:createActionStarfield".to_string();
    let found_func_for_all = all_funcs
        .iter()
        .find(|f| f.name == func_name)
        .expect("could not find func");
    let found_func_for_schema_variant = funcs_for_schema_variant
        .iter()
        .find(|f| f.name == func_name)
        .expect("could not find func");

    assert_eq!(found_func_for_all, found_func_for_schema_variant);
}

#[test]
async fn revertible(ctx: &mut DalContext) {
    // Find a func from builtins and create a new func.
    let func_id = Func::find_by_name(ctx, "test:createActionStarfield")
        .await
        .expect("could not perform find by name")
        .expect("func not found");
    let created_func = create_empty_action_func(ctx).await;

    // Before committing, ensure the existing func is revertible and that the new func is not revertible.
    let preexisting_func_is_revertible = Func::is_revertible_for_id(ctx, func_id)
        .await
        .expect("could not determine if revertible");
    let created_func_is_revertible = created_func
        .is_revertible(ctx)
        .await
        .expect("could not determine if revertible");
    assert!(preexisting_func_is_revertible);
    assert!(!created_func_is_revertible);

    // After committing, ensure the existing func is revertible and that the new func is not revertible.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    let preexisting_func_is_revertible = Func::is_revertible_for_id(ctx, func_id)
        .await
        .expect("could not determine if revertible");
    let created_func_is_revertible = created_func
        .is_revertible(ctx)
        .await
        .expect("could not determine if revertible");
    assert!(preexisting_func_is_revertible);
    assert!(!created_func_is_revertible);

    // Apply changes to the base change set.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Commit with the created func in the change set and create a new change set.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork change set");

    // In the forked change set, ensure the existing func is still revertible and that the new func
    // is too.
    let preexisting_func_is_revertible = Func::is_revertible_for_id(ctx, func_id)
        .await
        .expect("could not determine if revertible");
    let created_func_is_revertible = created_func
        .is_revertible(ctx)
        .await
        .expect("could not determine if revertible");
    assert!(preexisting_func_is_revertible);
    assert!(created_func_is_revertible);
}

#[test]
async fn duplicate(ctx: &mut DalContext) {
    let func_name = "Paul's Test Func".to_string();
    let authoring_func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Qualification,
        Some(func_name.clone()),
        None,
    )
    .await
    .expect("unable to create func");

    let func = Func::get_by_id_or_error(ctx, authoring_func.id)
        .await
        .expect("Unable to get the authored func");

    let duplicated_func_name = "Paul's Test Func Clone".to_string();
    let duplicated_func = func
        .duplicate(ctx, duplicated_func_name)
        .await
        .expect("Unable to duplicate the func");

    assert_eq!(duplicated_func.display_name, func.display_name);
    assert_eq!(duplicated_func.description, func.description);
    assert_eq!(duplicated_func.link, func.link);
    assert_eq!(duplicated_func.hidden, func.hidden);
    assert_eq!(duplicated_func.backend_kind, func.backend_kind);
    assert_eq!(
        duplicated_func.backend_response_type,
        func.backend_response_type
    );
    assert_eq!(duplicated_func.handler, func.handler);
    assert_eq!(duplicated_func.code_base64, func.code_base64);
}
