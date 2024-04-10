use dal::func::view::summary::FuncSummary;
use dal::{ChangeSet, DalContext, Func, Schema, SchemaVariant};
use dal_test::test;
use dal_test::test_harness::create_empty_action_func;
use pretty_assertions_sorted::assert_eq;

mod associations;

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
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
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
    ChangeSet::apply_to_base_change_set(ctx, true)
        .await
        .expect("could not apply to base change set");
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    // Commit with the created func in the change set and create a new change set.
    let new_change_set = ChangeSet::fork_head(ctx, "johnqt is the best igl")
        .await
        .expect("could not fork head");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility and snapshot to visibility");

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
