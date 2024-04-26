use dal::component::resource::ResourceView;
use dal::{
    Component, DalContext, DeprecatedAction, DeprecatedActionKind, DeprecatedActionPrototype,
};
use dal_test::helpers::create_component_for_schema_name;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn update_action(ctx: &mut DalContext) {
    let ms_swift = create_component_for_schema_name(ctx, "swifty", "ms swift").await;
    let swift_schema_variant_id = Component::schema_variant_id(ctx, ms_swift.id())
        .await
        .expect("Unable to get schema variant for component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

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

    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    let resource = ResourceView::get_by_component_id(ctx, ms_swift.id())
        .await
        .expect("unable to get the Resource view");

    if let Some(payload) = resource.payload {
        pretty_assertions_sorted::assert_eq!(serde_json::json![{"poop":true}], payload);
    } else {
        panic!("No resource data found for the component after create action");
    }

    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork change set");

    let actions_available = DeprecatedActionPrototype::for_variant(ctx, swift_schema_variant_id)
        .await
        .expect("Unable to get action prototypes");

    assert_eq!(4, actions_available.len());

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

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Apply to the base change set and commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    let queued_actions = DeprecatedAction::for_component(ctx, ms_swift.id())
        .await
        .expect("unable to list actions for component");
    assert!(queued_actions.is_empty());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let resource = ResourceView::get_by_component_id(ctx, ms_swift.id())
        .await
        .expect("unable to get the Resource view");

    if let Some(payload) = resource.payload {
        pretty_assertions_sorted::assert_eq!(serde_json::json![{"poonami":true}], payload);
    } else {
        panic!("No resource data found for the component after update action");
    }

    assert_eq!(
        1,
        ms_swift
            .attribute_values_for_prop(ctx, &["root", "resource", "last_synced"])
            .await
            .expect("should be able to get values")
            .len()
    );
}
