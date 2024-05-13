use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::{DalContext, Func, FuncId};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

mod attach;
mod attribute;

#[test]
async fn action(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = save_func_setup(ctx, "test:createActionStarfield").await;
}

#[test]
async fn authentication(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = save_func_setup(ctx, "test:setDummySecretString").await;
}

#[test]
async fn code_generation(ctx: &mut DalContext) {
    let (_func_id, _saved_func) = save_func_setup(ctx, "test:generateCode").await;
}

#[test]
async fn qualification(ctx: &mut DalContext) {
    let (_func_id, _saved_func) =
        save_func_setup(ctx, "test:qualificationDummySecretStringIsTodd").await;
}

// Sets up the tests within the module. Find the func to be saved by name and then save it
// immediately when found. This is the basic "does it work in place" check.
async fn save_func_setup(ctx: &mut DalContext, func_name: impl AsRef<str>) -> (FuncId, FuncView) {
    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let before = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");

    // Save the func immediately when found.
    FuncAuthoringClient::save_func(
        ctx,
        before.id,
        before.display_name,
        before.name,
        before.description,
        before.code,
        before.associations.clone(),
    )
    .await
    .expect("unable to save func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Perform base assertions before getting started.
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let after = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    assert_eq!(
        func_id,   // expected
        before.id  // actual
    );
    assert_eq!(
        before.types, // expected
        after.types,  // actual
    );

    (func_id, after)
}
