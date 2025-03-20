use dal::action::dependency_graph::ActionDependencyGraph;
use dal::component::frame::Frame;
use dal::{
    action::prototype::ActionKind, action::prototype::ActionPrototype, action::Action,
    action::ActionState, AttributeValue, Component, DalContext,
};
use dal_test::helpers::create_component_for_default_schema_name_in_default_view;
use dal_test::helpers::create_component_for_schema_name_with_type_on_default_view;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::helpers::{
    connect_components_with_socket_names, disconnect_components_with_socket_names,
};
use dal_test::{test, Result};
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn prototype_id(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
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
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
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
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
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
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
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
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
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
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "jack antonoff")
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

#[test]
async fn auto_queue_update(ctx: &mut DalContext) {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "jack antonoff")
            .await
            .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot to visibility");

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("apply changeset to base");

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx)
        .await
        .expect("deadline for actions to run exceeded");

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
    Component::enqueue_relevant_update_actions(ctx, av_id)
        .await
        .expect("could enqueue update func");

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
    assert_eq!(update_action_count, 1);
}

#[test]
async fn actions_are_ordered_correctly(ctx: &mut DalContext) {
    // create two components and connect them via edge
    let first_component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "small odd lego",
        "first component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");
    let second_component = create_component_for_schema_name_with_type_on_default_view(
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
    let third_component = create_component_for_schema_name_with_type_on_default_view(
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

#[test]
async fn simple_transitive_action_ordering(ctx: &mut DalContext) -> Result<()> {
    // Simple case: a chain of 3 components: A->B->C, an action for A and C only
    // First we'll connect the components via edges and ensure the create actions for A and C are ordered correctly
    // Next we'll remove the edges and replace them with Frames and ensure the actions are still ordered correctly
    // Finally we'll remove the create actions and enqueue Delete actions and ensure they're ordered correctly too!

    // create three components and connect them via edges
    let first_component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "small odd lego",
        "first component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await?;
    let first_component_id = first_component.id();
    let first_component_sv_id = first_component.schema_variant(ctx).await?.id();
    let second_component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "small even lego",
        "second component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await?;
    let second_component_id = second_component.id();

    connect_components_with_socket_names(
        ctx,
        first_component_id,
        "two",
        second_component_id,
        "two",
    )
    .await?;

    let third_component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "medium odd lego",
        "third component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await?;
    let third_component_id = third_component.id();
    let third_component_sv_id = third_component.schema_variant(ctx).await?.id();
    connect_components_with_socket_names(
        ctx,
        second_component_id,
        "one",
        third_component_id,
        "one",
    )
    .await?;
    // remove the action for the second component
    Action::remove_all_for_component_id(ctx, second_component_id).await?;

    // there should be two actions enqueued
    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component_id)
        .await
        .expect("could not get actions")
        .pop()
        .expect("doesn't have one");
    let third_component_actions = Action::find_for_component_id(ctx, third_component_id)
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");

    // Create for the third action is dependent on the create for the first action
    assert_eq!(
        action_graph.get_all_dependencies(first_component_action),
        vec![third_component_actions]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(third_component_actions),
        vec![first_component_action]
    );

    // now remove the edge and let's use frames
    disconnect_components_with_socket_names(
        ctx,
        first_component_id,
        "two",
        second_component_id,
        "two",
    )
    .await
    .expect("could not create connection");
    disconnect_components_with_socket_names(
        ctx,
        second_component_id,
        "one",
        third_component_id,
        "one",
    )
    .await
    .expect("could not create connection");
    Frame::upsert_parent(ctx, second_component_id, first_component_id).await?;
    Frame::upsert_parent(ctx, third_component_id, second_component_id).await?;

    // there should be two actions enqueued
    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    // order is still preserved
    let first_component_action = Action::find_for_component_id(ctx, first_component_id)
        .await
        .expect("could not get actions")
        .pop()
        .expect("doesn't have one");
    let third_component_actions = Action::find_for_component_id(ctx, third_component_id)
        .await
        .expect("could not list actions")
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");

    assert_eq!(
        action_graph.get_all_dependencies(first_component_action),
        vec![third_component_actions]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(third_component_actions),
        vec![first_component_action]
    );
    // now let's do destroy actions
    Action::remove_all_for_component_id(ctx, first_component_id).await?;
    Action::remove_all_for_component_id(ctx, third_component_id).await?;
    // there should be no actions enqueued
    let actions = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    // make sure two actions are enqueued
    assert!(actions.is_empty());
    // manually enqueue deletes for the first and third component
    let first_actions = ActionPrototype::for_variant(ctx, first_component_sv_id)
        .await
        .expect("could not list actions")
        .into_iter()
        .filter(|proto| proto.kind == ActionKind::Destroy)
        .collect_vec();
    assert_eq!(first_actions.len(), 1);

    let first_action = Action::new(ctx, first_actions[0].id, Some(first_component_id))
        .await?
        .id();

    let third_actions = ActionPrototype::for_variant(ctx, third_component_sv_id)
        .await?
        .into_iter()
        .filter(|proto| proto.kind == ActionKind::Destroy)
        .collect_vec();
    assert_eq!(third_actions.len(), 1);
    let third_action = Action::new(ctx, third_actions[0].id, Some(third_component_id))
        .await?
        .id();

    // there should be two actions enqueued
    let actions = Action::list_topologically(ctx).await?;
    assert_eq!(actions.len(), 2);

    let action_graph = ActionDependencyGraph::for_workspace(ctx)
        .await
        .expect("could not get graph");

    // Delete for the first action is dependent on the delete for the third action
    assert_eq!(
        action_graph.get_all_dependencies(third_action),
        vec![first_action]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(first_action),
        vec![third_action]
    );

    Ok(())
}
