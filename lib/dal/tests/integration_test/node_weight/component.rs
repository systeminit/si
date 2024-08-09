use dal::component::frame::Frame;
use dal::{Component, DalContext};
use dal_test::expected::{self, create_component_for_default_schema_name};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn component_can_only_have_one_parent(ctx: &mut DalContext) {
    let child_one =
        create_component_for_default_schema_name(ctx, "small even lego", "child component").await;

    let frame_one =
        create_component_for_default_schema_name(ctx, "small odd lego", "frame one").await;

    let frame_two =
        create_component_for_default_schema_name(ctx, "small odd lego", "frame two").await;

    expected::apply_change_set_to_base(ctx).await;

    for id in [child_one.id(), frame_one.id(), frame_two.id()] {
        Component::get_by_id(ctx, id)
            .await
            .expect("component should exist in head");
    }

    assert_eq!(
        None,
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("should not fail"),
        "should not have a parent at first"
    );

    let change_set_1 = expected::fork_from_head_change_set(ctx).await;
    Frame::upsert_parent(ctx, child_one.id(), frame_one.id())
        .await
        .expect("attach child to frame one");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(
        Some(frame_one.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    let _change_set_2 = expected::fork_from_head_change_set(ctx).await;
    Frame::upsert_parent(ctx, child_one.id(), frame_two.id())
        .await
        .expect("attach child to frame one");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    assert_eq!(
        Some(frame_two.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    // apply change set 2 to head
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(
        Some(frame_two.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    // switch to change set 1, and see that the component has been reparented
    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_1.id).await;
    assert_eq!(
        Some(frame_two.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    Frame::upsert_parent(ctx, child_one.id(), frame_one.id())
        .await
        .expect("reattach child to frame one");
    assert_eq!(
        Some(frame_one.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );
    // apply change set 1 to head
    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(
        Some(frame_one.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    // Both change sets have been applied, so they will stop receiving updates from head

    let change_set_3 = expected::fork_from_head_change_set(ctx).await;
    assert_eq!(
        Some(frame_one.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    // Move the child into frame two in change set 3
    Frame::upsert_parent(ctx, child_one.id(), frame_two.id())
        .await
        .expect("reattach child to frame one");

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    let _change_set_4 = expected::fork_from_head_change_set(ctx).await;
    assert_eq!(
        Some(frame_one.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    // Orphan child in change set 4
    Frame::orphan_child(ctx, child_one.id())
        .await
        .expect("able to orphan child");

    expected::apply_change_set_to_base(ctx).await;
    assert_eq!(
        None,
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    expected::update_visibility_and_snapshot_to_visibility(ctx, change_set_3.id).await;
    // despite being orphaned in change set 4, and head, we are not orphaned in
    // 3 because the 4 and head did not know about the frame relationship in 3
    assert_eq!(
        Some(frame_two.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );

    expected::apply_change_set_to_base(ctx).await;

    assert_eq!(
        Some(frame_two.id()),
        Component::get_parent_by_id(ctx, child_one.id())
            .await
            .expect("get parent by id succeeds")
    );
}
