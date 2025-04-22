use dal::{
    DalContext,
    Func,
    FuncId,
    func::authoring::FuncAuthoringClient,
};
use dal_test::{
    helpers::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_frontend_types::FuncSummary;

mod attribute;
mod detach;

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
pub async fn save_func_setup(
    ctx: &mut DalContext,
    func_name: impl AsRef<str>,
) -> (FuncId, FuncSummary) {
    let old_func_id = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("could not perform find func by name")
        .expect("no func found");

    let func_id = FuncAuthoringClient::create_unlocked_func_copy(ctx, old_func_id, None)
        .await
        .expect("could not create unlocked copy")
        .id;
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");
    let before = func
        .into_frontend_type(ctx)
        .await
        .expect("could not get func view");

    FuncAuthoringClient::update_func(ctx, func_id, Some("woo hoo".to_string()), None)
        .await
        .expect("could not save func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Perform base assertions before getting started.
    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("could not get func by id");
    let after = func
        .into_frontend_type(ctx)
        .await
        .expect("could not assemble func view");
    assert_eq!(
        func_id,        // expected
        before.func_id  // actual
    );
    assert_eq!(
        before.types, // expected
        after.types,  // actual
    );
    (func_id, after)
}
