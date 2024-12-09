use std::time::Duration;

use dal::action::prototype::ActionKind;
use dal::action::Action;
use dal::func::authoring::FuncAuthoringClient;
use dal::func::runner::FuncRunner;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::DalContext;
use dal_test::helpers::{
    create_component_for_schema_variant_on_default_view, ChangeSetTestHelpers,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use si_events::FuncRunState;

#[test]
async fn kill_execution_works(ctx: &mut DalContext) {
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "DOOM ETERNAL",
        None,
        None,
        "ID SOFTWARE",
        "#00b0b0",
    )
    .await
    .expect("could not create variant");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Add a new func to that asset that will be killed (it sleeps for awhile). After this, let's commit.
    let func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some("test:longAssCreateAction".to_string()),
        ActionKind::Create,
        variant.id(),
    )
    .await
    .expect("could new leaf func");
    let code = "async function main() {
        const ms = 600 * 1000;
        const sleep = new Promise((resolve) => setTimeout(resolve, ms));
        await sleep;
        return { payload: { \"poop\": true }, status: \"ok\" };
    }";
    FuncAuthoringClient::save_code(ctx, func.id, code.to_string())
        .await
        .expect("could not save code");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a new component for the new asset and commit.
    create_component_for_schema_variant_on_default_view(ctx, variant.id())
        .await
        .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Apply to the base change set and wait for all actions to run.
    assert!(ctx
        .parent_is_head()
        .await
        .expect("could not perform parent is head"));
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Find the action and its func run id.
    let mut action_ids = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    let action_id = action_ids.pop().expect("empty actions");
    assert!(action_ids.is_empty());

    // Wait for the func run to start.
    let mut maybe_func_run_id = None;
    for _ in 0..20 {
        maybe_func_run_id = ctx
            .layer_db()
            .func_run()
            .get_last_run_for_action_id(ctx.events_tenancy().workspace_pk, action_id)
            .await
            .expect("could not get last func run for action id")
            .map(|f| f.id());
        if maybe_func_run_id.is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let func_run_id = maybe_func_run_id.expect("no func run found");

    // Kill the active func run and observe that it worked.
    FuncRunner::kill_execution(ctx, func_run_id)
        .await
        .expect("could not kill execution");
    let func_run_state = ctx
        .layer_db()
        .func_run()
        .read(func_run_id)
        .await
        .expect("could not get func run")
        .expect("no func run found")
        .state();
    assert_eq!(
        FuncRunState::Killed, // expected
        func_run_state        // actual
    );
}
