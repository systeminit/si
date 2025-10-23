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
    component::{
        debug::ComponentDebugView,
        delete::delete_component,
    },
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
    // Create a component with both create and update actions and apply to head.
    let component_id = component::create(ctx, "swifty", "taylor swift").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    Action::remove_all_for_component_id(ctx, component_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    /////////////////

    // Create two change sets.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let update_change_set_id = ctx.change_set_id();
    assert!(
        dal::action::Action::list_topologically(ctx)
            .await?
            .is_empty()
    );
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let delete_change_set_id = ctx.change_set_id();
    assert!(
        dal::action::Action::list_topologically(ctx)
            .await?
            .is_empty()
    );

    // In the "delete" change set, erase the component.
    let component = Component::get_by_id(ctx, component_id).await?;
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, &[component_id]).await?;
    delete_component(ctx, &component, true, &head_components).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert!(
        dal::action::Action::list_topologically(ctx)
            .await?
            .is_empty()
    );

    // In the "update" chagne set, change the name, which auto-enqueues an update action.
    ctx.update_visibility_and_snapshot_to_visibility(update_change_set_id)
        .await?;
    assert!(
        dal::action::Action::list_topologically(ctx)
            .await?
            .is_empty()
    );
    value::set(ctx, ("taylor swift", "/si/name"), "travis kelce").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let mut actions = dal::action::Action::list_topologically(ctx).await?;
    assert_eq!(1, actions.len());
    let action_id = actions.pop().unwrap();
    let prototype = dal::action::Action::prototype(ctx, action_id).await?;
    assert_eq!(ActionKind::Update, prototype.kind);
    Action::set_state(ctx, action_id, ActionState::OnHold).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    ctx.update_visibility_and_snapshot_to_visibility(delete_change_set_id)
        .await?;
    // ChangeSetTestHelpers::switch_to_change_set(ctx, update_change_set_id).await?;

    // BUG! THIS SHOULD BE EMPTY.
    let action_ids = dal::action::Action::list_topologically(ctx).await?;
    dbg!("POOP", &action_ids);
    for action_id in action_ids {
        let action = dal::action::Action::get_by_id(ctx, action_id).await?;
        let prototype = dal::action::Action::prototype(ctx, action_id).await?;
        let component_id = Action::component_id(ctx, action_id).await?;
        dbg!("FOUND ACTION", &action, &prototype, &component_id);
    }
    assert!(false);

    Ok(())
}
