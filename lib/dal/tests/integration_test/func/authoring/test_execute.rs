use std::{
    sync::Arc,
    time::Duration,
};

use dal::{
    DalContext,
    Func,
    func::authoring::FuncAuthoringClient,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use si_db::FuncRunDb;
use si_events::{
    FuncRun,
    FuncRunId,
    FuncRunState,
};

#[test]
async fn test_execute_action_func(ctx: &mut DalContext) {
    let component_name = "Forty Six & 2";
    let func_name = "test:createActionStarfield";
    let func_args = serde_json::Value::Null;
    let schema_name = "starfield";

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");

    // Perform the test execution.
    let func_run_id =
        FuncAuthoringClient::test_execute_func(ctx, func_id, func_args, None, component.id())
            .await
            .expect("could not perform test execution for func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check the results.
    let func_run = wait_for_func_run_with_success_state(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
}

#[test]
async fn test_execute_attribute_func(ctx: &mut DalContext) {
    let component_name = "Jimmy";
    let func_name = "test:falloutEntriesToGalaxies";
    let func_args = serde_json::Value::Array(Vec::new());
    let schema_name = "starfield";

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");

    // Perform the test execution.
    let func_run_id =
        FuncAuthoringClient::test_execute_func(ctx, func_id, func_args, None, component.id())
            .await
            .expect("could not perform test execution for func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check the results.
    let func_run = wait_for_func_run_with_success_state(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
}

#[test]
async fn test_execute_code_generation_func(ctx: &mut DalContext) {
    let component_name = "Pushit";
    let func_name = "test:generateStringCode";
    let func_args = serde_json::value::Value::Null;
    let schema_name = "katy perry";

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");

    // Perform the test execution.
    let func_run_id =
        FuncAuthoringClient::test_execute_func(ctx, func_id, func_args, None, component.id())
            .await
            .expect("could not perform test execution for func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check the results.
    let func_run = wait_for_func_run_with_success_state(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
}

#[test]
async fn test_execute_qualification_func(ctx: &mut DalContext) {
    let component_name = "Third Eye";
    let func_name = "test:qualificationDummySecretStringIsTodd";
    let func_args = serde_json::value::Value::Null;
    let schema_name = "dummy-secret";

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");

    // Perform the test execution.
    let func_run_id =
        FuncAuthoringClient::test_execute_func(ctx, func_id, func_args, None, component.id())
            .await
            .expect("could not perform test execution for func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check the results.
    let func_run = wait_for_func_run_with_success_state(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
}

#[test]
async fn test_execute_with_modified_code(ctx: &mut DalContext) {
    let component_name = "Jimmy";
    let func_name = "test:falloutEntriesToGalaxies";
    let func_args = serde_json::Value::Array(Vec::new());
    let schema_name = "starfield";

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, schema_name, component_name)
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let old_func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func_id = FuncAuthoringClient::create_unlocked_func_copy(ctx, old_func_id, None)
        .await
        .expect("could create new func")
        .id;
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");

    // Perform the test execution with modified code.
    let modified_code = "async function falloutEntriesToGalaxies(input: Input): Promise<Output> { return [\"I was modified dammit!\"]; }";

    let func_run_id = FuncAuthoringClient::test_execute_func(
        ctx,
        func_id,
        func_args,
        Some(modified_code.to_string()),
        component.id(),
    )
    .await
    .expect("could not perform test execution for func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check the results.
    let func_run = wait_for_func_run_with_success_state(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
}

async fn wait_for_func_run_with_success_state(ctx: &DalContext, func_run_id: FuncRunId) -> FuncRun {
    let seconds = 15;

    for _ in 0..(seconds * 10) {
        let func_run = ctx
            .layer_db()
            .func_run()
            .read(func_run_id)
            .await
            .expect("could not read func run")
            .expect("func run not found");

        if func_run.state() == FuncRunState::Success {
            return Arc::unwrap_or_clone(func_run);
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    panic!("timed out waiting for func run");
}
