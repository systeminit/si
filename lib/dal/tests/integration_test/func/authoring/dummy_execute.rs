use dal::func::authoring::FuncAuthoringClient;
use dal::func::backend::js_action::DeprecatedActionRunResult;
use dal::func::view::FuncView;
use dal::{DalContext, Func};
use dal_test::helpers::{create_component_for_schema_name, ChangeSetTestHelpers};
use dal_test::test;
use veritech_client::ResourceStatus;

#[test]
async fn dummy_execute_action_func(ctx: &mut DalContext) {
    let component_name = "Just a Girl";
    let execution_key = "No Doubt";
    let func_name = "test:createActionStarfield";
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
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    let result = FuncAuthoringClient::dummy_execute_func(
        ctx,
        func_view.id,
        serde_json::Value::Null,
        execution_key.to_string(),
        func_view.code.expect("no code found"),
        component.id(),
    )
    .await
    .expect("could not perform dummy execution for func");

    assert_eq!(
        func_id,   // expected
        result.id  // actual
    );
    assert_eq!(
        execution_key,                 // expected
        result.execution_key.as_str()  // actual
    );
    let action_run_result: DeprecatedActionRunResult =
        serde_json::from_value(result.output).expect("could not deserialize");
    assert_eq!(
        ResourceStatus::Ok,                              // expected
        action_run_result.status.expect("empty status")  // actual
    );
}

#[test]
async fn dummy_execute_attribute_func(ctx: &mut DalContext) {
    let component_name = "Pushit";
    let execution_key = "Tool";
    let func_name = "test:falloutEntriesToGalaxies";
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
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    let result = FuncAuthoringClient::dummy_execute_func(
        ctx,
        func_view.id,
        serde_json::Value::Array(Vec::new()),
        execution_key.to_string(),
        func_view.code.expect("no code found"),
        component.id(),
    )
    .await
    .expect("could not perform dummy execution for func");

    assert_eq!(
        func_id,   // expected
        result.id  // actual
    );
    assert_eq!(
        execution_key,                 // expected
        result.execution_key.as_str()  // actual
    );

    // TODO(nick): we should ensure that the dummy execution did not affect the actual component.
}
