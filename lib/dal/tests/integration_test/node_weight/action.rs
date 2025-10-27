use std::collections::HashSet;

use dal::{
    self,
    Component,
    ComponentId,
    DalContext,
    action::{
        Action,
        prototype::ActionKind,
    },
    component::delete::delete_component,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        component,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_events::ActionState;

#[test]
async fn prevent_orphaned_update_actions(ctx: &mut DalContext) -> Result<()> {
    // Create a component with both create and update actions and apply to HEAD.
    let component_id = component::create(ctx, "swifty", "taylor swift").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    // Make sure we start with a clean slate.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    Action::remove_all_for_component_id(ctx, component_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    // Create two change sets needed to reproduce the issue.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let alpha_change_set_id = ctx.change_set_id();
    assert!(Action::list_topologically(ctx).await?.is_empty());
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let beta_change_set_id = ctx.change_set_id();
    assert!(Action::list_topologically(ctx).await?.is_empty());

    // In the "beta", change set, erase the component.
    let component = Component::get_by_id(ctx, component_id).await?;
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, &[component_id]).await?;
    delete_component(ctx, &component, true, &head_components).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert!(Action::list_topologically(ctx).await?.is_empty());

    // In the "alpha" change set, change the component name, which auto-enqueues an update action.
    ctx.update_visibility_and_snapshot_to_visibility(alpha_change_set_id)
        .await?;
    assert!(Action::list_topologically(ctx).await?.is_empty());
    value::set(ctx, ("taylor swift", "/si/name"), "travis kelce").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let mut actions = Action::list_topologically(ctx).await?;
    assert_eq!(1, actions.len());
    let action_id = actions.pop().unwrap();
    let prototype = Action::prototype(ctx, action_id).await?;
    assert_eq!(ActionKind::Update, prototype.kind);

    // Put the action "OnHold" to ensure it stays on HEAD after apply. Then, apply the change set to HEAD.
    Action::set_state(ctx, action_id, ActionState::OnHold).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    // Switch to the "beta" change set and ensure that the action does not exist. Before the change
    // introduced alongside this test, the action would exist in this change set, but not point to
    // a component. In the UI, the action would looked "orphaned" to the end user.
    ctx.update_visibility_and_snapshot_to_visibility(beta_change_set_id)
        .await?;
    assert!(Action::list_topologically(ctx).await?.is_empty());

    Ok(())
}
