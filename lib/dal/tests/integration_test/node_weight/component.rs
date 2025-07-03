use std::{
    collections::HashSet,
    time::Duration,
};

use dal::{
    Component,
    ComponentId,
    DalContext,
    component::frame::Frame,
    func::authoring::FuncAuthoringClient,
};
use dal_test::{
    expected::{
        self,
        ExpectComponent,
        apply_change_set_to_base,
        commit_and_update_snapshot_to_visibility,
        fork_from_head_change_set,
        update_visibility_and_snapshot_to_visibility,
    },
    helpers::{
        attribute::value,
        connect_components_with_socket_names,
        get_component_input_socket_value,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_events::FuncRun;

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

    tokio::time::sleep(Duration::from_secs(2)).await;

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

    assert!(
        !ctx.workspace_snapshot()
            .expect("get snap")
            .node_exists(docker_image_1.id())
            .await
    );
}

#[test]
async fn deleting_a_connected_component_doesnt_cause_nonconnected_components_to_process(
    ctx: &mut DalContext,
) {
    let docker_image_1 = ExpectComponent::create_named(ctx, "Docker Image", "docker 1").await;
    let docker_image_1_id = docker_image_1.id();
    let docker_image_2 = ExpectComponent::create_named(ctx, "Docker Image", "docker 2").await;

    let butane_1 = ExpectComponent::create_named(ctx, "Butane", "butane 1")
        .await
        .component(ctx)
        .await;
    let butane_1_id = butane_1.id();
    let butane_2 = ExpectComponent::create_named(ctx, "Butane", "butane 2")
        .await
        .component(ctx)
        .await;

    // connect both pairs of Docker -> Butane
    connect_components_with_socket_names(
        ctx,
        docker_image_1.id(),
        "Container Image",
        butane_1.id(),
        "Container Image",
    )
    .await
    .expect("able to connect");
    connect_components_with_socket_names(
        ctx,
        docker_image_2.id(),
        "Container Image",
        butane_2.id(),
        "Container Image",
    )
    .await
    .expect("able to connect");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    expected::apply_change_set_to_base(ctx).await;
    let prop_path = &["root", "domain", "systemd", "units"];
    // open 2 new change sets
    let cs_1 = fork_from_head_change_set(ctx).await;

    // ensure this butane component has a value for the input socket
    let butane_1_input_socket =
        get_component_input_socket_value(ctx, butane_1_id, "Container Image")
            .await
            .expect("couldn't find value");

    // get the attribute value id for the second butane component to check when it's function was last run
    assert!(butane_1_input_socket.is_some());
    let mut attribute_value_ids = butane_2
        .attribute_values_for_prop(ctx, prop_path)
        .await
        .expect("couldn't find attribute values");
    let attribute_value_id = attribute_value_ids.pop().expect("has an attribute value");

    // I'm not actually looking for a qualification but there's nothing qualification specific here anyways
    // using this to validate that this func didn't re-run unnecessarily after removing the docker image
    // connected to the other butane component
    let func_run_pre_delete: Option<FuncRun> = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.workspace_pk().expect("has a workspace"),
            attribute_value_id,
        )
        .await
        .expect("could not get func run");
    // remove one of the connected docker images
    let docker = Component::get_by_id(ctx, docker_image_1_id)
        .await
        .expect("couldn't get component");
    docker.delete(ctx).await.expect("couldn't delete");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // ensure the butane component no longer has value for that input socket
    let butane_1_after_commit =
        get_component_input_socket_value(ctx, butane_1_id, "Container Image")
            .await
            .expect("couldn't find value");

    assert!(butane_1_after_commit.is_none());

    // ensure the func that sets the prop for butane_2 didn't rerun (it should have the same func_run_id!)

    let func_run_post_delete: Option<FuncRun> = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.workspace_pk().expect("has a workspace"),
            attribute_value_id,
        )
        .await
        .expect("could not get func run");

    assert_eq!(
        func_run_post_delete.as_ref().expect("has a value").id(),
        func_run_pre_delete.expect("has a value").id()
    );

    // before we apply, create a second fork of head
    let cs_2 = fork_from_head_change_set(ctx).await;
    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

    // now apply
    expected::apply_change_set_to_base(ctx).await;

    update_visibility_and_snapshot_to_visibility(ctx, cs_2.id).await;

    //now switch to cs_2 and ensure the butane_1 component is updated correctly but the butane_2 component is as it was
    // and we haven't re-ran the existing input socket
    let func_run_post_apply: Option<FuncRun> = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.workspace_pk().expect("has a workspace"),
            attribute_value_id,
        )
        .await
        .expect("could not get func run");
    assert_eq!(
        func_run_post_delete.expect("has a value").id(),
        func_run_post_apply.expect("has a value").id()
    );

    let butane_1_after_apply =
        get_component_input_socket_value(ctx, butane_1_id, "Container Image")
            .await
            .expect("couldn't find value");

    assert!(butane_1_after_apply.is_none());
}

/// THIS TEST DOESN'T PASS 
#[test]
async fn deleting_a_connected_component_doesnt_cause_nonconnected_components_to_process_subscriptions(
    ctx: &mut DalContext,
) {
    let docker_image_1 = ExpectComponent::create_named(ctx, "Docker Image", "docker 1").await;
    let docker_image_1_id = docker_image_1.id();
    let docker_image_2 = ExpectComponent::create_named(ctx, "Docker Image", "docker 2").await;

    let butane_1 = ExpectComponent::create_named(ctx, "Butane", "butane 1")
        .await
        .component(ctx)
        .await;
    let butane_2 = ExpectComponent::create_named(ctx, "Butane", "butane 2")
        .await
        .component(ctx)
        .await;

    // Use subscriptions instead of socket connections
    // butane components subscribe to docker image values

    let func = FuncAuthoringClient::create_new_transformation_func(
        ctx,
        Some("transform_to_units".to_string()),
    )
    .await
    .expect("couldn't create new func");
    let code = "async function main(input: Input): Promise < Output > {
        if (input === undefined || input === null) return {};
         
            let unit: Record < string, any > = {
                name: input.image + '.service',
                enabled: true,
            };

            let ports = '';
            let dockerImageExposedPorts = input.ExposedPorts;
            if (
                !(
                    dockerImageExposedPorts === undefined ||
                    dockerImageExposedPorts === null
                )
            ) {
                dockerImageExposedPorts.forEach(function(dockerImageExposedPort: any) {
                    if (
                        !(
                            dockerImageExposedPort === undefined ||
                            dockerImageExposedPort === null
                        )
                    ) {
                        let parts = dockerImageExposedPort.split('/');
                        try {
                            // Prefix with a blank space.
                            ports = ports + ` --publish ${parts[0]}:${parts[0]}`;
                        } catch (err) {}
                    }
                });
            }

            let image = input.image;
            let defaultDockerHost = 'docker.io';
            let imageParts = image.split('/');
            if (imageParts.length === 1) {
                image = [defaultDockerHost, 'library', imageParts[0]].join('/');
            } else if (imageParts.length === 2) {
                image = [defaultDockerHost, imageParts[0], imageParts[1]].join('/');
            }

            let description = name.charAt(0).toUpperCase() + name.slice(1);

            unit.contents = `[Unit]\nDescription=${description}\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill ${name}\nExecStartPre=-/bin/podman rm ${name}\nExecStartPre=/bin/podman pull ${image}\nExecStart=/bin/podman run --name ${name}${ports} ${image}\n\n[Install]\nWantedBy=multi-user.target`;
        return unit;
    }";

    FuncAuthoringClient::save_code(ctx, func.id, code)
        .await
        .expect("couldn't save code");
    // first create an entry in the domain/units array
    // then subscribe that entry to docker image with a transformation function?

    value::subscribe_with_custom_function(
        ctx,
        (butane_1.id(), "/domain/systemd/units"),
        [(docker_image_1.id(), "/domain")],
        Some(func.id),
    )
    .await
    .expect("able to create subscription");

    value::subscribe_with_custom_function(
        ctx,
        (butane_2.id(), "/domain/systemd/units"),
        [(docker_image_2.id(), "/domain")],
        Some(func.id),
    )
    .await
    .expect("able to create subscription");
    value::set(
        ctx,
        (docker_image_2.id(), "/domain/image"),
        "systeminit/whiskers",
    )
    .await
    .expect("couldn't set value");
    value::set(
        ctx,
        (docker_image_1.id(), "/domain/image"),
        "systeminit/whiskers",
    )
    .await
    .expect("couldn't set image");

    expected::commit_and_update_snapshot_to_visibility(ctx).await;
    let monster = value::get(ctx, (butane_2.id(), "/domain/systemd/units"))
        .await
        .expect("couldn't get value");
    dbg!(&monster);

    // I think this is failing for the same reason here - so not able to actually test the expected scenario
    let val_2 = value::get(ctx, (butane_2.id(), "/domain/systemd/units/0"))
        .await
        .expect("couldn't get value");
    let val_1 = value::get(ctx, (butane_1.id(), "/domain/systemd/units/0"))
        .await
        .expect("couldn't get value");
    dbg!(&val_1);
    dbg!(&val_2);
    expected::apply_change_set_to_base(ctx).await;
    let prop_path = &["root", "domain", "systemd", "units", "0"];
    // open 2 new change sets
    let cs_1 = fork_from_head_change_set(ctx).await;

    // get the attribute value id for the second butane component to check when it's function was last run
    let mut attribute_value_ids = butane_2
        .attribute_values_for_prop(ctx, prop_path)
        .await
        .expect("couldn't find attribute values");
    let attribute_value_id = attribute_value_ids.pop().expect("has an attribute value");

    // I'm not actually looking for a qualification but there's nothing qualification specific here anyways
    // using this to validate that this func didn't re-run unnecessarily after removing the docker image
    // connected to the other butane component
    let func_run_pre_delete: Option<FuncRun> = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.workspace_pk().expect("has a workspace"),
            attribute_value_id,
        )
        .await
        .expect("could not get func run");

    // remove one of the connected docker images
    let docker = Component::get_by_id(ctx, docker_image_1_id)
        .await
        .expect("couldn't get component");
    docker.delete(ctx).await.expect("couldn't delete");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // ensure the func that sets the prop for butane_2 didn't rerun (it should have the same func_run_id!)
    let func_run_post_delete: Option<FuncRun> = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.workspace_pk().expect("has a workspace"),
            attribute_value_id,
        )
        .await
        .expect("could not get func run");

    assert_eq!(
        func_run_post_delete.as_ref().expect("has a value").id(),
        func_run_pre_delete.expect("has a value").id()
    );

    // before we apply, create a second fork of head
    let cs_2 = fork_from_head_change_set(ctx).await;
    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

    // now apply
    expected::apply_change_set_to_base(ctx).await;

    update_visibility_and_snapshot_to_visibility(ctx, cs_2.id).await;

    //now switch to cs_2 and ensure the butane_1 component is updated correctly but the butane_2 component is as it was
    // and we haven't re-ran the existing input socket
    let func_run_post_apply: Option<FuncRun> = ctx
        .layer_db()
        .func_run()
        .get_last_qualification_for_attribute_value_id(
            ctx.workspace_pk().expect("has a workspace"),
            attribute_value_id,
        )
        .await
        .expect("could not get func run");
    assert_eq!(
        func_run_post_delete.expect("has a value").id(),
        func_run_post_apply.expect("has a value").id()
    );
}

#[ignore]
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
