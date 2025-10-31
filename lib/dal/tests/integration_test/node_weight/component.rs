use dal::{
    AttributePrototype,
    Component,
    DalContext,
    attribute::value::debug::AttributeDebugView,
    component::debug::ComponentDebugView,
};
use dal_test::{
    Result,
    expected::{
        self,
        ExpectComponent,
        apply_change_set_to_base,
        fork_from_head_change_set,
        update_visibility_and_snapshot_to_visibility,
    },
    helpers::{
        ChangeSetTestHelpers,
        attribute::value::{
            self,
            AttributeValueKey,
        },
        component,
    },
    test,
};

#[test]
async fn deleting_a_component_deletes_component_in_other_change_sets(ctx: &mut DalContext) {
    let docker_image_1 = ExpectComponent::create_named(ctx, "Docker Image", "docker 1").await;
    expected::apply_change_set_to_base(ctx).await;

    // fork this change set and place docker image in a frame
    let cs_1 = fork_from_head_change_set(ctx).await;
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
async fn deleting_a_value_source_component_does_not_break_other_change_sets(
    ctx: &mut DalContext,
) -> Result<()> {
    // Create two components with an inter-component subscription.
    let component_to_erase_id = component::create(ctx, "Docker Image", "hammerfell").await?;
    value::set(ctx, ("hammerfell", "/domain/image"), "neloth").await?;

    // Apply to HEAD
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // In a new change set, create a component connected to the component_to_erase
    let subscriber_cs = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    let subscriber_component_id = component::create(ctx, "Docker Image", "highrock").await?;
    value::subscribe(
        ctx,
        ("highrock", "/domain/image"),
        ("hammerfell", "/domain/image"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        "neloth",                                              // expected
        value::get(ctx, ("highrock", "/domain/image")).await?, // actual
    );

    // Now, in a second change set, erase `component_to_erase` and apply to head
    Component::remove(ctx, component_to_erase_id).await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    ChangeSetTestHelpers::switch_to_change_set(ctx, subscriber_cs.id).await?;

    let subscriber_av_id = AttributeValueKey::id(ctx, ("highrock", "/domain/image")).await?;

    // ensure debug view does not fail
    let _component_debug_view = ComponentDebugView::new(ctx, subscriber_component_id).await?;
    let debug_view = AttributeDebugView::new(ctx, subscriber_av_id, None, None).await?;

    let prototype_id = debug_view.prototype_id.expect("prototype found");

    let apas = AttributePrototype::list_arguments(ctx, prototype_id).await?;
    assert!(apas.is_empty());

    // Ensure we can build the component's mvs
    dal_materialized_views::component::assemble(ctx.clone(), subscriber_component_id).await?;
    dal_materialized_views::component::attribute_tree::assemble(
        ctx.clone(),
        subscriber_component_id,
    )
    .await?;

    Ok(())
}

// TODO restore this using subscriptions!
// #[test]
// async fn deleting_a_connected_component_doesnt_cause_nonconnected_components_to_process(
//     ctx: &mut DalContext,
// ) {
//     let docker_image_1 = ExpectComponent::create_named(ctx, "Docker Image", "docker 1").await;
//     let docker_image_1_id = docker_image_1.id();
//     let docker_image_2 = ExpectComponent::create_named(ctx, "Docker Image", "docker 2").await;

//     let butane_1 = ExpectComponent::create_named(ctx, "Butane", "butane 1")
//         .await
//         .component(ctx)
//         .await;
//     let butane_1_id = butane_1.id();
//     let butane_2 = ExpectComponent::create_named(ctx, "Butane", "butane 2")
//         .await
//         .component(ctx)
//         .await;

//     // connect both pairs of Docker -> Butane
//     connect_components_with_socket_names(
//         ctx,
//         docker_image_1.id(),
//         "Container Image",
//         butane_1.id(),
//         "Container Image",
//     )
//     .await
//     .expect("able to connect");
//     connect_components_with_socket_names(
//         ctx,
//         docker_image_2.id(),
//         "Container Image",
//         butane_2.id(),
//         "Container Image",
//     )
//     .await
//     .expect("able to connect");
//     expected::commit_and_update_snapshot_to_visibility(ctx).await;

//     expected::apply_change_set_to_base(ctx).await;
//     let prop_path = &["root", "domain", "systemd", "units"];
//     // open 2 new change sets
//     let cs_1 = fork_from_head_change_set(ctx).await;

//     // ensure this butane component has a value for the input socket
//     let butane_1_input_socket =
//         get_component_input_socket_value(ctx, butane_1_id, "Container Image")
//             .await
//             .expect("couldn't find value");

//     // get the attribute value id for the second butane component to check when it's function was last run
//     assert!(butane_1_input_socket.is_some());
//     let mut attribute_value_ids = butane_2
//         .attribute_values_for_prop(ctx, prop_path)
//         .await
//         .expect("couldn't find attribute values");
//     let attribute_value_id = attribute_value_ids.pop().expect("has an attribute value");

//     // I'm not actually looking for a qualification but there's nothing qualification specific here anyways
//     // using this to validate that this func didn't re-run unnecessarily after removing the docker image
//     // connected to the other butane component
//     let func_run_pre_delete: Option<FuncRun> = ctx
//         .layer_db()
//         .func_run()
//         .get_last_qualification_for_attribute_value_id(
//             ctx.workspace_pk().expect("has a workspace"),
//             attribute_value_id,
//         )
//         .await
//         .expect("could not get func run");
//     // remove one of the connected docker images
//     let docker = Component::get_by_id(ctx, docker_image_1_id)
//         .await
//         .expect("couldn't get component");
//     docker.delete(ctx).await.expect("couldn't delete");
//     expected::commit_and_update_snapshot_to_visibility(ctx).await;

//     // ensure the butane component no longer has value for that input socket
//     let butane_1_after_commit =
//         get_component_input_socket_value(ctx, butane_1_id, "Container Image")
//             .await
//             .expect("couldn't find value");

//     assert!(butane_1_after_commit.is_none());

//     // ensure the func that sets the prop for butane_2 didn't rerun (it should have the same func_run_id!)

//     let func_run_post_delete: Option<FuncRun> = ctx
//         .layer_db()
//         .func_run()
//         .get_last_qualification_for_attribute_value_id(
//             ctx.workspace_pk().expect("has a workspace"),
//             attribute_value_id,
//         )
//         .await
//         .expect("could not get func run");

//     assert_eq!(
//         func_run_post_delete.as_ref().expect("has a value").id(),
//         func_run_pre_delete.expect("has a value").id()
//     );

//     // before we apply, create a second fork of head
//     let cs_2 = fork_from_head_change_set(ctx).await;
//     update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

//     // now apply
//     expected::apply_change_set_to_base(ctx).await;

//     update_visibility_and_snapshot_to_visibility(ctx, cs_2.id).await;

//     //now switch to cs_2 and ensure the butane_1 component is updated correctly but the butane_2 component is as it was
//     // and we haven't re-ran the existing input socket
//     let func_run_post_apply: Option<FuncRun> = ctx
//         .layer_db()
//         .func_run()
//         .get_last_qualification_for_attribute_value_id(
//             ctx.workspace_pk().expect("has a workspace"),
//             attribute_value_id,
//         )
//         .await
//         .expect("could not get func run");
//     assert_eq!(
//         func_run_post_delete.expect("has a value").id(),
//         func_run_post_apply.expect("has a value").id()
//     );

//     let butane_1_after_apply =
//         get_component_input_socket_value(ctx, butane_1_id, "Container Image")
//             .await
//             .expect("couldn't find value");

//     assert!(butane_1_after_apply.is_none());
// }
