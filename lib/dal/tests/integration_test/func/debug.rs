use dal::{
    DalContext,
    Func,
    func::runner::FuncRunner,
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
