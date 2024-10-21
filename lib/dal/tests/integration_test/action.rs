use dal::action::dependency_graph::ActionDependencyGraph;
use dal::component::frame::Frame;
use dal::{
    action::prototype::ActionKind, action::prototype::ActionPrototype, action::Action,
    action::ActionState, AttributeValue, Component, DalContext,
};
use dal_test::helpers::create_component_for_default_schema_name;
use dal_test::helpers::create_component_for_schema_name_with_type;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::helpers::{
    connect_components_with_socket_names, disconnect_components_with_socket_names,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn prototype_id(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "swifty", "shake it off")
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut action = None;
    let mut prototype = None;
    for proto in ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if proto.kind == ActionKind::Create {
            action = Some(
                Action::new(ctx, proto.id, Some(component.id()))
                    .await
                    .expect("unable to upsert action"),
            );
            prototype = Some(proto);
            break;
        }
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        Action::prototype_id(ctx, action.expect("no action found").id())
            .await
            .expect("unable to find prototype"),
        prototype.expect("unable to find prototype").id()
    );
}

#[test]
async fn component(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "swifty", "shake it off")
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut action = None;
    for prototype in ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == ActionKind::Create {
            action = Some(
                Action::new(ctx, prototype.id, Some(component.id()))
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        Action::component_id(ctx, action.expect("no action found").id())
            .await
            .expect("unable to find component"),
        Some(component.id())
    );
}

#[test]
async fn get_by_id(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "swifty", "shake it off")
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let mut action = None;
    for prototype in ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
    {
        if prototype.kind == ActionKind::Create {
            action = Some(
                Action::new(ctx, prototype.id, Some(component.id()))
                    .await
                    .expect("unable to upsert action"),
            );
            break;
        }
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let action = action.expect("no action found");
    assert_eq!(
        Action::get_by_id(ctx, action.id())
            .await
            .expect("unable to get action"),
        action
    );
}

#[test]
async fn set_state(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "swifty", "shake it off")
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let prototypes = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant");
    assert!(!prototypes.is_empty());
    for prototype in prototypes {
        if prototype.kind == ActionKind::Create {
            let action = Action::new(ctx, prototype.id, Some(component.id()))
                .await
                .expect("unable to upsert action");
            assert_eq!(action.state(), ActionState::Queued);

            Action::set_state(ctx, action.id(), ActionState::Running)
                .await
                .expect("unable to set state");

            let action = Action::get_by_id(ctx, action.id())
                .await
                .expect("unable to get action by id");
            assert_eq!(action.state(), ActionState::Running);
            break;
        }
    }
}

#[test]
async fn run(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "swifty", "shake it off")
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let (maybe_resource, _func_run_id) = ActionPrototype::run(ctx, proto.id(), component.id())
        .await
        .expect("unable to run ActionPrototype");
    assert!(maybe_resource.is_some());
}

#[test]
async fn auto_queue_creation(ctx: &mut DalContext) {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component = create_component_for_default_schema_name(ctx, "swifty", "jack antonoff")
        .await
        .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let action_ids = Action::list_topologically(ctx)
        .await
        .expect("find action ids");
    assert_eq!(action_ids.len(), 1);

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id)
            .await
            .expect("find action by id");
        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id)
                .await
                .expect("get prototype id from action");
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id)
                .await
                .expect("get prototype from id");

            assert_eq!(prototype.kind, ActionKind::Create);
        }
    }

    // ======================================================
    // Deleting a component with no resource should dequeue the creation action
    // ======================================================
    component.delete(ctx).await.expect("delete component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let action_ids = Action::list_topologically(ctx)
        .await
        .expect("find action ids");

    assert!(action_ids.is_empty());
}

// TODO This test is a stub that should be fixed after actions v2 is done
// Right now, the workspace for tests does not have the actions flag set so this won't yield any results
// The tests cases are valid
#[test]
async fn auto_queue_update_and_destroy(ctx: &mut DalContext) {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component = create_component_for_default_schema_name(ctx, "swifty", "jack antonoff")
        .await
        .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot to visibility");

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("apply changeset to base");

    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("fork from head");

    // ======================================================
    // Updating values in a component that has a resource should enqueue an update action
    // ======================================================

    let name_path = &["root", "si", "name"];
    let av_id = component
        .attribute_values_for_prop(ctx, name_path)
        .await
        .expect("find value ids for the prop treasure")
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("whomever")))
        .await
        .expect("override domain/name attribute value");

    let action_ids = Action::list_topologically(ctx)
        .await
        .expect("find action ids");

    let mut update_action_count = 0;

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id)
            .await
            .expect("find action by id");

        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id)
                .await
                .expect("get prototype id from action");
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id)
                .await
                .expect("get action prototype by id");

            if prototype.kind == ActionKind::Update {
                update_action_count += 1;
            };
        }
    }

    // TODO: fix this, update actions have been disabled for now so they wont be automatically enqueued
    // As they were being enqueued in the wrong place in AttributeValue, causing actions to be enqueued and immediately run by DVU's running on headg
    assert_eq!(update_action_count, 0);

    // ======================================================
    // Deleting a component with resource should queue the Destroy action
    // ======================================================
    component.delete(ctx).await.expect("delete component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // TODO Fix the following section
    // Since the creation action never actually runs on the test (or at least we can't wait for it)
    // The resource never gets created. A Destroy action only gets queued
    // (implicitly by component.delete above) if the component has a resource,
    // So the check below is failing

    // let action_ids = Action::list_topologically(ctx)
    //     .await
    //     .expect("find action ids");
    //
    // let mut deletion_action_count = 0;
    // for action_id in action_ids {
    //     let action = dbg!(Action::get_by_id(ctx, action_id)
    //         .await
    //         .expect("find action by id"));
    //     if action.state() == ActionState::Queued {
    //         let prototype_id = Action::prototype_id(ctx, action_id)
    //             .await
    //             .expect("get prototype id from action");
    //         let prototype = ActionPrototype::get_by_id(ctx, prototype_id)
    //             .await
    //             .expect("get action prototype by id");
    //
    //         if prototype.kind == ActionKind::Destroy {
    //             deletion_action_count += 1;
    //         }
    //     }
    // }

    // assert_eq!(deletion_action_count, 1);
}

#[test]
async fn actions_are_ordered_correctly(ctx: &mut DalContext) {
    // create two components and connect them via edge
    let first_component = create_component_for_schema_name_with_type(
        ctx,
        "small odd lego",
        "first component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");
    let second_component = create_component_for_schema_name_with_type(
        ctx,
        "small even lego",
        "second component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    connect_components_with_socket_names(
        ctx,
        first_component.id(),
        "two",
        second_component.id(),
        "two",
    )
    .await
    .expect("could not create connection");

    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await
        .expect("could not get actions")
        .pop()
        .expect("doesn't have one");
    let second_component_actions = Action::find_for_component_id(ctx, second_component.id())
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");

    dbg!(action_graph.get_all_dependencies(first_component_action));
    dbg!(action_graph.get_all_dependencies(second_component_actions));
    dbg!(action_graph.direct_dependencies_of(first_component_action));
    dbg!(action_graph.direct_dependencies_of(second_component_actions));
    assert_eq!(
        action_graph.get_all_dependencies(first_component_action),
        vec![second_component_actions]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(second_component_actions),
        vec![first_component_action]
    );
    // now let's remove the connection and put the second component inside the first
    disconnect_components_with_socket_names(
        ctx,
        first_component.id(),
        "two",
        second_component.id(),
        "two",
    )
    .await
    .expect("could not create connection");
    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await
        .expect("could not get actions")
        .pop()
        .expect("doesn't have one");
    let second_component_actions = Action::find_for_component_id(ctx, second_component.id())
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");

    assert_eq!(
        action_graph.get_all_dependencies(first_component_action),
        vec![]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(second_component_actions),
        vec![]
    );

    Frame::upsert_parent(ctx, second_component.id(), first_component.id())
        .await
        .expect("could not upsert");
    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await
        .expect("could not get actions")
        .pop()
        .expect("doesn't have one");
    let second_component_actions = Action::find_for_component_id(ctx, second_component.id())
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");

    assert_eq!(
        action_graph.get_all_dependencies(first_component_action),
        vec![second_component_actions]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(second_component_actions),
        vec![first_component_action]
    );

    // now let's add another component and draw an edge.
    let third_component = create_component_for_schema_name_with_type(
        ctx,
        "small even lego",
        "third component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    connect_components_with_socket_names(
        ctx,
        first_component.id(),
        "two",
        third_component.id(),
        "two",
    )
    .await
    .expect("could not create connection");

    // make sure actions are ordered correctly
    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");

    // make sure three actions are enqueued now
    assert_eq!(actions.len(), 3);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await
        .expect("could not get actions")
        .pop()
        .expect("doesn't have one");
    let second_component_action = Action::find_for_component_id(ctx, second_component.id())
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let third_component_action = Action::find_for_component_id(ctx, third_component.id())
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");
    let dependencies = action_graph.get_all_dependencies(first_component_action);
    assert_eq!(dependencies.len(), 2);
    assert!(dependencies.contains(&second_component_action));
    assert!(dependencies.contains(&third_component_action));
    assert_eq!(
        action_graph.direct_dependencies_of(second_component_action),
        vec![first_component_action]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(third_component_action),
        vec![first_component_action]
    );
}
