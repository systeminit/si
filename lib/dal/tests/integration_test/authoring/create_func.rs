use dal::func::{authoring, FuncKind};
use dal::{ChangeSet, DalContext, Func};
use dal_test::test;
use dal_test::test_harness::commit_and_update_snapshot;

#[test]
async fn create_qualification_no_options(ctx: &mut DalContext) {
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let func_name = "Paul's Test Func".to_string();
    let func = authoring::create_func(ctx, FuncKind::Qualification, Some(func_name.clone()), None)
        .await
        .expect("unable to create func");

    commit_and_update_snapshot(ctx).await;

    assert_eq!(FuncKind::Qualification, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    result: 'success',\n    message: 'Component qualified'\n  };\n}\n".to_string()),  func.code);

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}
