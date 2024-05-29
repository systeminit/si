use dal::func::authoring::FuncAuthoringClient;
use dal::{DalContext, Func};
use dal_test::helpers::{create_component_for_schema_name, ChangeSetTestHelpers};
use dal_test::test;
use si_events::{FuncRun, FuncRunId, FuncRunLog, FuncRunState};
use std::sync::Arc;
use std::time::Duration;

#[test]
async fn test_execute_action_func(ctx: &mut DalContext) {
    let component_name = "Forty Six & 2";
    let func_name = "test:createActionStarfield";
    let func_args = serde_json::Value::Null;
    let schema_name = "starfield";

    let component = create_component_for_schema_name(ctx, schema_name, component_name).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
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
    let (func_run, func_run_log) = wait_for_func_run(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
    let mut logs = func_run_log.logs().to_vec();
    let log = logs.pop().expect("empty logs");
    assert!(logs.is_empty());
    assert_eq!(
        "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"ayrtonsennajscommand\",\n  \"payload\": {\n    \"poop\": true\n  },\n  \"health\": \"ok\"\n}", // expected
        log.message.as_str(), // actual
    );
}

#[test]
async fn test_execute_attribute_func(ctx: &mut DalContext) {
    let component_name = "Jimmy";
    let func_name = "test:falloutEntriesToGalaxies";
    let func_args = serde_json::Value::Array(Vec::new());
    let schema_name = "starfield";

    let component = create_component_for_schema_name(ctx, schema_name, component_name).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
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
    let (func_run, func_run_log) = wait_for_func_run(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
    let mut logs = func_run_log.logs().to_vec();
    let log = logs.pop().expect("empty logs");
    assert!(logs.is_empty());
    assert_eq!(
        "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"tomcruise\",\n  \"data\": [],\n  \"unset\": false\n}", // expected
        log.message.as_str(), // actual
    );
}

#[test]
async fn test_execute_code_generation_func(ctx: &mut DalContext) {
    let component_name = "Pushit";
    let func_name = "test:generateStringCode";
    let func_args = serde_json::value::Value::Null;
    let schema_name = "katy perry";

    let component = create_component_for_schema_name(ctx, schema_name, component_name).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
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
    let (func_run, func_run_log) = wait_for_func_run(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
    let mut logs = func_run_log.logs().to_vec();
    let log = logs.pop().expect("empty logs");
    assert!(logs.is_empty());
    assert_eq!(
        "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"tomcruise\",\n  \"data\": {\n    \"format\": \"string\",\n    \"code\": \"poop canoe\"\n  },\n  \"unset\": false\n}", // expected
        log.message.as_str(), // actual
    );
}

#[test]
async fn test_execute_qualification_func(ctx: &mut DalContext) {
    let component_name = "Third Eye";
    let func_name = "test:qualificationDummySecretStringIsTodd";
    let func_args = serde_json::value::Value::Null;
    let schema_name = "dummy-secret";

    let component = create_component_for_schema_name(ctx, schema_name, component_name).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
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
    let (func_run, func_run_log) = wait_for_func_run(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
    let mut logs = func_run_log.logs().to_vec();
    let log = logs.pop().expect("empty logs");
    assert!(logs.is_empty());
    assert_eq!(
        "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"tomcruise\",\n  \"data\": {\n    \"result\": \"failure\",\n    \"message\": \"dummy secret string is empty\"\n  },\n  \"unset\": false\n}", // expected
        log.message.as_str(), // actual
    );
}

#[test]
async fn test_execute_with_modified_code(ctx: &mut DalContext) {
    let component_name = "Jimmy";
    let func_name = "test:falloutEntriesToGalaxies";
    let func_args = serde_json::Value::Array(Vec::new());
    let schema_name = "starfield";

    let component = create_component_for_schema_name(ctx, schema_name, component_name).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");

    // Perform the test execution with modified code.
    let modified_code =
        "async function falloutEntriesToGalaxies(input: Input): Promise<Output> { return [\"I was modified dammit!\"]; }";
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
    let (func_run, func_run_log) = wait_for_func_run(ctx, func_run_id).await;
    assert_eq!(
        func.name.as_str(),       // expected
        func_run.function_name()  // actual
    );
    let mut logs = func_run_log.logs().to_vec();
    let log = logs.pop().expect("empty logs");
    assert!(logs.is_empty());
    assert_eq!(
        "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"tomcruise\",\n  \"data\": [\n    \"I was modified dammit!\"\n  ],\n  \"unset\": false\n}", // expected
        log.message.as_str(), // actual
    );
}

async fn wait_for_func_run(ctx: &DalContext, func_run_id: FuncRunId) -> (FuncRun, FuncRunLog) {
    let func_run = wait_for_func_run_only(ctx, func_run_id)
        .await
        .expect("timeout while waiting for successful func run");
    let func_run_log = wait_for_func_run_log_only(ctx, func_run_id)
        .await
        .expect("timeout while waiting for finalized func run log");
    (func_run, func_run_log)
}

async fn wait_for_func_run_only(ctx: &DalContext, func_run_id: FuncRunId) -> Option<FuncRun> {
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
            return Some(Arc::unwrap_or_clone(func_run));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    None
}

async fn wait_for_func_run_log_only(
    ctx: &DalContext,
    func_run_id: FuncRunId,
) -> Option<FuncRunLog> {
    let seconds = 15;

    for _ in 0..(seconds * 10) {
        let func_run_log = ctx
            .layer_db()
            .func_run_log()
            .get_for_func_run_id(func_run_id)
            .await
            .expect("could not read func run log")
            .expect("func run log not found");

        if func_run_log.is_finalized() {
            return Some(Arc::unwrap_or_clone(func_run_log));
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    None
}
