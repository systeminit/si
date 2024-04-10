use dal::component::resource::ResourceView;
use dal::{
    ChangeSet, Component, DalContext, DeprecatedAction, DeprecatedActionKind,
    DeprecatedActionPrototype,
};
use dal_test::test;
use dal_test::test_harness::{commit_and_update_snapshot, create_component_for_schema_name};

#[test]
async fn update_action(ctx: &mut DalContext) {
    let ms_swift = create_component_for_schema_name(ctx, "swifty", "ms swift").await;
    let swift_schema_variant_id = Component::schema_variant_id(ctx, ms_swift.id())
        .await
        .expect("Unable to get schema variant for component");

    commit_and_update_snapshot(ctx).await;

    let mut actions = DeprecatedAction::for_component(ctx, ms_swift.id())
        .await
        .expect("unable to list actions for component");
    pretty_assertions_sorted::assert_eq!(
        1,             // expected
        actions.len()  // actual
    );
    let create_action = actions.pop().expect("no actions found");
    let create_action_prototype = create_action
        .prototype(ctx)
        .await
        .expect("could not get action prototype for action");
    pretty_assertions_sorted::assert_eq!(
        DeprecatedActionKind::Create, // expected
        create_action_prototype.kind, // actual
    );

    assert!(ctx
        .parent_is_head()
        .await
        .expect("could not perform parent is head"));

    let applied_change_set = ChangeSet::apply_to_base_change_set(ctx, true)
        .await
        .expect("could apply to base change set");
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(
        applied_change_set
            .base_change_set_id
            .expect("base change set not found"),
    )
    .await
    .expect("could not update visibility and snapshot to visibility");

    let resource = ResourceView::get_by_component_id(ctx, ms_swift.id())
        .await
        .expect("unable to get the Resource view");

    if let Some(payload) = resource.data {
        pretty_assertions_sorted::assert_eq!(serde_json::json![{"poop":true}], payload);
    } else {
        panic!("No resource data found for the component after create action");
    }

    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let actions_available = DeprecatedActionPrototype::for_variant(ctx, swift_schema_variant_id)
        .await
        .expect("Unable to get action prototypes");

    assert_eq!(3, actions_available.len());

    let mut update_actions: Vec<&DeprecatedActionPrototype> = actions_available
        .iter()
        .filter(|a| a.kind == DeprecatedActionKind::Other)
        .collect();
    assert_eq!(1, update_actions.len());

    let action = update_actions.pop().expect("Unable to get the action");

    DeprecatedAction::upsert(ctx, action.id, ms_swift.id())
        .await
        .expect("Unable to insert an action");

    let mut queued_actions = DeprecatedAction::for_component(ctx, ms_swift.id())
        .await
        .expect("unable to list actions for component");
    pretty_assertions_sorted::assert_eq!(
        1,                    // expected
        queued_actions.len()  // actual
    );
    let action_detail = queued_actions.pop().expect("no actions found");
    let action_prototype = action_detail
        .prototype(ctx)
        .await
        .expect("could not get action prototype for action");
    pretty_assertions_sorted::assert_eq!(
        DeprecatedActionKind::Other, // expected
        action_prototype.kind,       // actual
    );

    commit_and_update_snapshot(ctx).await;

    // Apply to the base change set and commit.
    let applied_change_set = ChangeSet::apply_to_base_change_set(ctx, true)
        .await
        .expect("could apply to base change set");
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(
        applied_change_set
            .base_change_set_id
            .expect("base change set not found"),
    )
    .await
    .expect("could not update visibility and snapshot to visibility");

    let queued_actions = DeprecatedAction::for_component(ctx, ms_swift.id())
        .await
        .expect("unable to list actions for component");
    assert!(queued_actions.is_empty());

    commit_and_update_snapshot(ctx).await;

    let resource = ResourceView::get_by_component_id(ctx, ms_swift.id())
        .await
        .expect("unable to get the Resource view");

    if let Some(payload) = resource.data {
        pretty_assertions_sorted::assert_eq!(serde_json::json![{"poonami":true}], payload);
    } else {
        panic!("No resource data found for the component after update action");
    }
}
