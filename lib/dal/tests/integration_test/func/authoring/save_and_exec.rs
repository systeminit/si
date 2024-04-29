use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::{DalContext, Func};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn save_and_exec_action_func(ctx: &mut DalContext) {
    let func_name = "test:createActionStarfield";
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

    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        func_view.associations,
    )
    .await
    .expect("could not save func");

    FuncAuthoringClient::execute_func(ctx, func_view.id)
        .await
        .expect("could not execute func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
}

#[test]
async fn save_and_exec_attribute_func(ctx: &mut DalContext) {
    let func_name = "test:falloutEntriesToGalaxies";
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

    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        func_view.associations,
    )
    .await
    .expect("could not save func");

    FuncAuthoringClient::execute_func(ctx, func_view.id)
        .await
        .expect("could not execute func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
}
