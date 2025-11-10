use std::time::Duration;

use dal::{
    DalContext,
    Func,
    func::{
        debug::{
            DebugFuncJobState,
            DebugFuncJobStateRow,
            dispatch_debug_func,
        },
        runner::FuncRunner,
    },
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use veritech_client::ComponentKind;

#[test]
async fn test_execute_debug_func(ctx: &mut DalContext) {
    let code = r#"function debug({ component, debugInput }) {
        console.log('Debugging');

        return { a: component.properties?.domain?.name, b: debugInput };
    }"#;
    let debug_func = Func::new_debug("debug_test", code, "debug");

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "debug it")
            .await
            .expect("make debug component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot");

    let properties = component.view(ctx).await.expect("get component view");
    let args = serde_json::json!({ "debug_input": "test",  "component": {
        "kind": ComponentKind::Standard,
        "properties": properties.clone(),
        "id": component.id(),
    }  });

    let (_func_run_id, func_run) = FuncRunner::run_debug(ctx, debug_func, component.id(), args)
        .await
        .expect("run debug func");

    let mut run_value = func_run
        .await
        .expect("get run value")
        .expect("is sucessful");

    let value = run_value.take_value().expect("should have a value");

    assert_eq!("debug it", value["output"]["a"]);
    assert_eq!("test", value["output"]["b"]);
}

#[test]
async fn test_debug_func_job(ctx: &mut DalContext) {
    let code = r#"function debug({ component, debugInput }) {
        console.log('Debugging');

        return { a: component.properties?.domain?.name, b: debugInput };
    }"#;
    let debug_func = Func::new_debug("debug_test", code, "debug");

    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "debug it")
            .await
            .expect("make debug component");

    let debug_input = serde_json::json!({
       "ged": "sparrowhawk",
    });

    let job_state_id = dispatch_debug_func(ctx, component.id(), debug_func, Some(debug_input))
        .await
        .expect("dispatch debug func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot");

    let mut job_state;
    let max = 10_000;
    let mut count = 0;
    loop {
        job_state = DebugFuncJobStateRow::get_by_id(ctx, job_state_id)
            .await
            .expect("get job state");

        if job_state.state == DebugFuncJobState::Success {
            break;
        }

        count += 1;
        if count > max {
            panic!("timeout");
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let expected_output = serde_json::json!({
        "a": "debug it",
        "b": { "ged": "sparrowhawk" }
    });

    assert_eq!(
        expected_output,
        job_state.result.expect("should have a value")["output"]
    );
}
