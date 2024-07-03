use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::{DalContext, Func};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn save_and_exec_action_func(ctx: &mut DalContext) {
    let func_name = "test:createActionStarfield";
    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");

    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could not create unlocked copy");
    FuncAuthoringClient::update_func(ctx, new_func.id, Some("woo hoo".to_string()), None)
        .await
        .expect("could not update func");

    FuncAuthoringClient::execute_func(ctx, new_func.id)
        .await
        .expect("could not execute func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
}

#[test]
async fn save_and_exec_attribute_func(ctx: &mut DalContext) {
    let func_name = "test:falloutEntriesToGalaxies";
    let func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could not create unlocked copy");
    FuncAuthoringClient::update_func(ctx, new_func.id, Some("woo hoo".to_string()), None)
        .await
        .expect("could not update func");

    FuncAuthoringClient::execute_func(ctx, new_func.id)
        .await
        .expect("could not execute func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
}
