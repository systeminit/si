use std::collections::HashSet;

use dal::component::frame::Frame;
use dal::{Component, ComponentId, DalContext};
use dal_test::expected::{
    self, apply_change_set_to_base, commit_and_update_snapshot_to_visibility,
    fork_from_head_change_set, update_visibility_and_snapshot_to_visibility, ExpectComponent,
};
use dal_test::helpers::connect_components_with_socket_names;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn component_can_only_have_one_parent(ctx: &mut DalContext) {
    let child_one = ExpectComponent::create_named(ctx, "small even lego", "child component").await;
    let frame_one = ExpectComponent::create_named(ctx, "small odd lego", "frame one").await;
    let frame_two = ExpectComponent::create_named(ctx, "small odd lego", "frame two").await;

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

#[test]
async fn deleting_a_component_deletes_component_in_other_change_sets(ctx: &mut DalContext) {
    let docker_image_1 = ExpectComponent::create_named(ctx, "Docker Image", "docker 1").await;
    expected::apply_change_set_to_base(ctx).await;

    // fork this change set and place docker image in a frame
    let cs_1 = fork_from_head_change_set(ctx).await;
    let frame_one = ExpectComponent::create_named(ctx, "small odd lego", "frame one").await;
    Frame::upsert_parent(ctx, docker_image_1.id(), frame_one.id())
        .await
        .expect("attach child to frame one");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // fork and delete a docker image
    fork_from_head_change_set(ctx).await;
    Component::remove(ctx, docker_image_1.id())
        .await
        .expect("able to remove");

    apply_change_set_to_base(ctx).await;

    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

    assert!(ctx
        .workspace_snapshot()
        .expect("get snap")
        .get_node_index_by_id_opt(docker_image_1.id())
        .await
        .is_none());
}

#[test]
async fn deleting_a_component_deletes_outgoing_connections_in_other_change_sets(
    ctx: &mut DalContext,
) {
    let docker_image_1 = ExpectComponent::create_named(ctx, "Docker Image", "docker 1").await;
    let docker_image_2 = ExpectComponent::create_named(ctx, "Docker Image", "docker 2").await;
    let docker_image_3 = ExpectComponent::create_named(ctx, "Docker Image", "docker 3").await;
    let docker_image_4 = ExpectComponent::create_named(ctx, "Docker Image", "docker 4").await;

    let butane = ExpectComponent::create_named(ctx, "Butane", "butane")
        .await
        .component(ctx)
        .await;

    expected::apply_change_set_to_base(ctx).await;

    // fork this change set and connect the docker images to the butane
    let cs_1 = fork_from_head_change_set(ctx).await;
    let mut docker_ids = HashSet::from([
        docker_image_1.id(),
        docker_image_2.id(),
        docker_image_3.id(),
        docker_image_4.id(),
    ]);

    for docker_image_id in &docker_ids {
        connect_components_with_socket_names(
            ctx,
            *docker_image_id,
            "Container Image",
            butane.id(),
            "Container Image",
        )
        .await
        .expect("able to connect")
    }
    commit_and_update_snapshot_to_visibility(ctx).await;
    let incoming_sources: HashSet<ComponentId> = butane
        .incoming_connections(ctx)
        .await
        .expect("able to get incoming connections")
        .iter()
        .map(|conn| conn.from_component_id)
        .collect();

    assert_eq!(docker_ids, incoming_sources);

    // fork and delete a docker image
    fork_from_head_change_set(ctx).await;
    Component::remove(ctx, docker_image_1.id())
        .await
        .expect("able to remove");
    docker_ids.remove(&docker_image_1.id());

    apply_change_set_to_base(ctx).await;

    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;
    let incoming_sources: HashSet<ComponentId> = butane
        .incoming_connections(ctx)
        .await
        .expect("able to get incoming connections")
        .iter()
        .map(|conn| conn.from_component_id)
        .collect();

    assert_eq!(docker_ids, incoming_sources);

    // fork and delete the rest
    fork_from_head_change_set(ctx).await;
    for docker_id in docker_ids {
        Component::remove(ctx, docker_id)
            .await
            .expect("delete delete");
    }
    apply_change_set_to_base(ctx).await;

    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;
    let incoming_sources: HashSet<ComponentId> = butane
        .incoming_connections(ctx)
        .await
        .expect("able to get incoming connections")
        .iter()
        .map(|conn| conn.from_component_id)
        .collect();
    assert!(incoming_sources.is_empty());
}
