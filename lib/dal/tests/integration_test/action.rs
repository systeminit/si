use dal::{
    AttributeValue,
    Component,
    DalContext,
    action::{
        Action,
        ActionState,
        dependency_graph::ActionDependencyGraph,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    component::frame::Frame,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        component,
        connect_components_with_socket_names,
        create_component_for_default_schema_name_in_default_view,
        create_component_for_schema_name_with_type_on_default_view,
        disconnect_components_with_socket_names,
    },
    test,
};
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;
use si_id::ActionId;

#[test]
async fn prototype_id(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;
    let mut action = None;
    let mut prototype = None;
    for proto in ActionPrototype::for_variant(ctx, variant_id).await? {
        if proto.kind == ActionKind::Create {
            action = Some(Action::new(ctx, proto.id, Some(component.id())).await?);
            prototype = Some(proto);
            break;
        }
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        Action::prototype_id(ctx, action.expect("no action found").id()).await?,
        prototype.expect("unable to find prototype").id()
    );

    Ok(())
}

#[test]
async fn component(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;
    let mut action = None;
    for prototype in ActionPrototype::for_variant(ctx, variant_id).await? {
        if prototype.kind == ActionKind::Create {
            action = Some(Action::new(ctx, prototype.id, Some(component.id())).await?);
            break;
        }
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        Action::component_id(ctx, action.expect("no action found").id()).await?,
        Some(component.id())
    );

    Ok(())
}

#[test]
async fn get_by_id(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;
    let mut action = None;
    for prototype in ActionPrototype::for_variant(ctx, variant_id).await? {
        if prototype.kind == ActionKind::Create {
            action = Some(Action::new(ctx, prototype.id, Some(component.id())).await?);
            break;
        }
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action = action.expect("no action found");
    assert_eq!(Action::get_by_id(ctx, action.id()).await?, action);

    Ok(())
}

#[test]
async fn set_state(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;
    let prototypes = ActionPrototype::for_variant(ctx, variant_id).await?;
    assert!(!prototypes.is_empty());
    for prototype in prototypes {
        if prototype.kind == ActionKind::Create {
            let action = Action::new(ctx, prototype.id, Some(component.id())).await?;
            assert_eq!(action.state(), ActionState::Queued);

            Action::set_state(ctx, action.id(), ActionState::Running).await?;

            let action = Action::get_by_id(ctx, action.id()).await?;
            assert_eq!(action.state(), ActionState::Running);
            break;
        }
    }

    Ok(())
}

#[test]
async fn run(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await?
        .pop()
        .expect("unable to find prototype for variant");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let (maybe_resource, _func_run_id) =
        ActionPrototype::run(ctx, proto.id(), component.id()).await?;
    assert!(maybe_resource.is_some());

    Ok(())
}

#[test]
async fn auto_queue_creation(ctx: &mut DalContext) -> Result<()> {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "jack antonoff")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::list_topologically(ctx).await?;
    assert_eq!(action_ids.len(), 1);

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;
        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;

            assert_eq!(prototype.kind, ActionKind::Create);
        }
    }

    // ======================================================
    // Deleting a component with no resource should dequeue the creation action
    // ======================================================
    component.delete(ctx).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    assert!(action_ids.is_empty());

    Ok(())
}

#[test]
async fn auto_queue_update(ctx: &mut DalContext) -> Result<()> {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component_jack =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "jack antonoff")
            .await?;
    let component_swift =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "taylor swift")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    // ======================================================
    // Updating values in a component that has a resource should enqueue an update action
    // ======================================================

    let name_path = &["root", "si", "name"];
    let av_id = component_jack
        .attribute_values_for_prop(ctx, name_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    // Note: we're updating the root/si/name - which propagates to root/domain/name
    // and as this component has a resource, DVU should be enqueuing the update func!
    AttributeValue::update(ctx, av_id, Some(serde_json::json!("whomever"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    let mut update_action_count = 0;

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;

        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
            let component_id = Action::component_id(ctx, action_id)
                .await?
                .expect("is some");
            if prototype.kind == ActionKind::Update && component_id == component_jack.id() {
                update_action_count += 1;
            };
        }
    }
    assert_eq!(update_action_count, 1);

    // ======================================================
    // Updating values in a component that has a resource should not enqueue an update
    // action if the value didn't change
    // ======================================================
    let name_path = &["root", "si", "name"];
    let av_id = component_swift
        .attribute_values_for_prop(ctx, name_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("taylor swift"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    let mut update_action_count = 0;

    for action_id in &action_ids {
        let action_id = *action_id;
        let action = Action::get_by_id(ctx, action_id).await?;
        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
            let component_id = Action::component_id(ctx, action_id)
                .await?
                .expect("is some");
            if prototype.kind == ActionKind::Update && component_id == component_swift.id() {
                update_action_count += 1;
            };
        }
    }
    // didn't actually change the value, so there should not be an update function for swifty!
    assert_eq!(update_action_count, 0);

    Ok(())
}

#[test]
async fn actions_are_ordered_correctly(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect them via edge
    let first_component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "small odd lego",
        "first component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await?;
    let second_component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "small even lego",
        "second component",
        dal::ComponentType::ConfigurationFrameDown,
    )
    .await?;

    connect_components_with_socket_names(
        ctx,
        first_component.id(),
        "two",
        second_component.id(),
        "two",
    )
    .await?;

    let actions = Action::list_topologically(ctx).await?;
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await?
        .pop()
        .expect("doesn't have one");
    let second_component_actions = Action::find_for_component_id(ctx, second_component.id())
        .await?
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;

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
    .await?;
    let actions = Action::list_topologically(ctx).await?;
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await?
        .pop()
        .expect("doesn't have one");
    let second_component_actions = Action::find_for_component_id(ctx, second_component.id())
        .await?
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;

    assert_eq!(
        action_graph.get_all_dependencies(first_component_action),
        vec![]
    );
    assert_eq!(
        action_graph.direct_dependencies_of(second_component_actions),
        vec![]
    );

    Frame::upsert_parent(ctx, second_component.id(), first_component.id()).await?;
    let actions = Action::list_topologically(ctx).await?;
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await?
        .pop()
        .expect("doesn't have one");
    let second_component_actions = Action::find_for_component_id(ctx, second_component.id())
        .await?
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;

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
    .await?;

    connect_components_with_socket_names(
        ctx,
        first_component.id(),
        "two",
        third_component.id(),
        "two",
    )
    .await?;

    // make sure actions are ordered correctly
    let actions = Action::list_topologically(ctx).await?;

    // make sure three actions are enqueued now
    assert_eq!(actions.len(), 3);

    let first_component_action = Action::find_for_component_id(ctx, first_component.id())
        .await?
        .pop()
        .expect("doesn't have one");
    let second_component_action = Action::find_for_component_id(ctx, second_component.id())
        .await?
        .pop()
        .expect("didnt have an action");
    let third_component_action = Action::find_for_component_id(ctx, third_component.id())
        .await?
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;
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

    Ok(())
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
    let actions = Action::list_topologically(ctx).await?;
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    let first_component_action = Action::find_for_component_id(ctx, first_component_id)
        .await?
        .pop()
        .expect("doesn't have one");
    let third_component_actions = Action::find_for_component_id(ctx, third_component_id)
        .await?
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;

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
    .await?;
    disconnect_components_with_socket_names(
        ctx,
        second_component_id,
        "one",
        third_component_id,
        "one",
    )
    .await?;
    Frame::upsert_parent(ctx, second_component_id, first_component_id).await?;
    Frame::upsert_parent(ctx, third_component_id, second_component_id).await?;

    // there should be two actions enqueued
    let actions = Action::list_topologically(ctx).await?;
    // make sure two actions are enqueued
    assert_eq!(actions.len(), 2);

    // order is still preserved
    let first_component_action = Action::find_for_component_id(ctx, first_component_id)
        .await?
        .pop()
        .expect("doesn't have one");
    let third_component_actions = Action::find_for_component_id(ctx, third_component_id)
        .await?
        .pop()
        .expect("didnt have an action");
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;

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
    let actions = Action::list_topologically(ctx).await?;
    // make sure two actions are enqueued
    assert!(actions.is_empty());
    // manually enqueue deletes for the first and third component
    let first_actions = ActionPrototype::for_variant(ctx, first_component_sv_id)
        .await?
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

    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;

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

#[test]
async fn create_action_ordering_subscriptions(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    assert_eq!(
        vec!["Create a", "Create b", "Create c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    value::subscribe(ctx, (b, "/domain/two"), [(a, "/domain/two")]).await?;
    assert_eq!(vec!["Create a", "Create c"], next_actions(ctx).await?);
    value::subscribe(ctx, (c, "/domain/one"), [(b, "/domain/one")]).await?;
    assert_eq!(vec!["Create a"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn create_action_ordering_sockets(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    assert_eq!(
        vec!["Create a", "Create b", "Create c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    connect_components_with_socket_names(ctx, a, "two", b, "two").await?;
    assert_eq!(vec!["Create a", "Create c"], next_actions(ctx).await?);
    connect_components_with_socket_names(ctx, b, "one", c, "one").await?;
    assert_eq!(vec!["Create a"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn create_action_ordering_mixed(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    assert_eq!(
        vec!["Create a", "Create b", "Create c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    connect_components_with_socket_names(ctx, a, "two", b, "two").await?;
    assert_eq!(vec!["Create a", "Create c"], next_actions(ctx).await?);
    value::subscribe(ctx, (c, "/domain/one"), [(b, "/domain/one")]).await?;
    assert_eq!(vec!["Create a"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn create_action_ordering_mixed_2(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    assert_eq!(
        vec!["Create a", "Create b", "Create c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    value::subscribe(ctx, (b, "/domain/two"), [(a, "/domain/two")]).await?;
    assert_eq!(vec!["Create a", "Create c"], next_actions(ctx).await?);
    connect_components_with_socket_names(ctx, b, "one", c, "one").await?;
    assert_eq!(vec!["Create a"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn delete_action_ordering_subscriptions(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    Action::remove_all_for_component_id(ctx, a).await?;
    Action::remove_all_for_component_id(ctx, b).await?;
    Action::remove_all_for_component_id(ctx, c).await?;
    enqueue_delete_action(ctx, a).await?;
    enqueue_delete_action(ctx, b).await?;
    enqueue_delete_action(ctx, c).await?;
    assert_eq!(
        vec!["Destroy a", "Destroy b", "Destroy c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    value::subscribe(ctx, (b, "/domain/two"), [(a, "/domain/two")]).await?;
    assert_eq!(vec!["Destroy b", "Destroy c"], next_actions(ctx).await?);
    value::subscribe(ctx, (c, "/domain/one"), [(b, "/domain/one")]).await?;
    assert_eq!(vec!["Destroy c"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn delete_action_ordering_sockets(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    Action::remove_all_for_component_id(ctx, a).await?;
    Action::remove_all_for_component_id(ctx, b).await?;
    Action::remove_all_for_component_id(ctx, c).await?;
    enqueue_delete_action(ctx, a).await?;
    enqueue_delete_action(ctx, b).await?;
    enqueue_delete_action(ctx, c).await?;
    assert_eq!(
        vec!["Destroy a", "Destroy b", "Destroy c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    connect_components_with_socket_names(ctx, a, "two", b, "two").await?;
    assert_eq!(vec!["Destroy b", "Destroy c"], next_actions(ctx).await?);
    connect_components_with_socket_names(ctx, b, "one", c, "one").await?;
    assert_eq!(vec!["Destroy c"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn delete_action_ordering_mixed(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    Action::remove_all_for_component_id(ctx, a).await?;
    Action::remove_all_for_component_id(ctx, b).await?;
    Action::remove_all_for_component_id(ctx, c).await?;
    enqueue_delete_action(ctx, a).await?;
    enqueue_delete_action(ctx, b).await?;
    enqueue_delete_action(ctx, c).await?;
    assert_eq!(
        vec!["Destroy a", "Destroy b", "Destroy c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    connect_components_with_socket_names(ctx, a, "two", b, "two").await?;
    assert_eq!(vec!["Destroy b", "Destroy c"], next_actions(ctx).await?);
    value::subscribe(ctx, (c, "/domain/one"), [(b, "/domain/one")]).await?;
    assert_eq!(vec!["Destroy c"], next_actions(ctx).await?);

    Ok(())
}

#[test]
async fn delete_action_ordering_mixed_2(ctx: &mut DalContext) -> Result<()> {
    // create two components and connect a.two -> b.two and b.two -> c.two via subscriptions
    let a = component::create(ctx, "small odd lego", "a").await?;
    let b = component::create(ctx, "small even lego", "b").await?;
    let c = component::create(ctx, "medium odd lego", "c").await?;
    Action::remove_all_for_component_id(ctx, a).await?;
    Action::remove_all_for_component_id(ctx, b).await?;
    Action::remove_all_for_component_id(ctx, c).await?;
    enqueue_delete_action(ctx, a).await?;
    enqueue_delete_action(ctx, b).await?;
    enqueue_delete_action(ctx, c).await?;
    assert_eq!(
        vec!["Destroy a", "Destroy b", "Destroy c"],
        next_actions(ctx).await?
    );

    // Now check that the actions are ordered correctly
    value::subscribe(ctx, (b, "/domain/two"), [(a, "/domain/two")]).await?;
    assert_eq!(vec!["Destroy b", "Destroy c"], next_actions(ctx).await?);
    connect_components_with_socket_names(ctx, b, "one", c, "one").await?;
    assert_eq!(vec!["Destroy c"], next_actions(ctx).await?);

    Ok(())
}

async fn next_actions(ctx: &mut DalContext) -> Result<Vec<String>> {
    let mut result = vec![];
    let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;
    for action_id in action_graph.independent_actions() {
        let component_id = Action::component_id(ctx, action_id)
            .await?
            .expect("component");
        result.push(format!(
            "{} {}",
            Action::prototype(ctx, action_id).await?.kind,
            Component::name_by_id(ctx, component_id).await?,
        ));
    }
    result.sort();
    Ok(result)
}

async fn enqueue_delete_action(
    ctx: &mut DalContext,
    component_id: dal::ComponentId,
) -> Result<ActionId> {
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let action = ActionPrototype::for_variant(ctx, variant_id)
        .await?
        .into_iter()
        .find(|proto| proto.kind == ActionKind::Destroy)
        .expect("no destroy action found");
    Ok(Action::new(ctx, action.id, Some(component_id)).await?.id())
}
