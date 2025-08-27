use std::time::Duration;

use dal::{
    AttributeValue,
    ChangeSet,
    Component,
    DalContext,
    SchemaVariant,
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
    func::authoring::FuncAuthoringClient,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    expected::ExpectSchemaVariant,
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
use pretty_assertions_sorted::{
    assert_eq,
    assert_ne,
};
use serde_json::json;
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

    let mut jack_actions = Action::find_for_component_id(ctx, component_jack.id()).await?;
    assert_eq!(1, jack_actions.len());
    let jack_action_id = jack_actions.pop().expect("no action found");
    let jack_action = Action::get_by_id(ctx, jack_action_id).await?;
    assert_eq!(ActionState::Failed, jack_action.state());

    let name_path = &["root", "si", "name"];
    let av_id = component_jack
        .attribute_values_for_prop(ctx, name_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    // ======================================================
    // Updating values in a Component that has a Failed action should not enqueue an update
    // ======================================================

    // Note: we're updating the root/si/name - which propagates to root/domain/name
    // and as this component has a resource, DVU should be enqueuing the update func!
    AttributeValue::update(ctx, av_id, Some(serde_json::json!("nope"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::find_for_component_id(ctx, component_jack.id()).await?;
    assert!(!action_ids.is_empty());
    let mut found_failed_action = false;
    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;
        assert_ne!(
            ActionKind::Update,
            Action::prototype(ctx, action.id()).await?.kind
        );
        if action.state() == ActionState::Failed {
            found_failed_action = true;
        }
    }
    assert!(found_failed_action);

    // ======================================================
    // Updating values in a Component that has a Queued action should not enqueue an update
    // ======================================================
    for action_id in Action::find_for_component_id(ctx, component_jack.id()).await? {
        Action::set_state(ctx, action_id, ActionState::Queued).await?;
    }

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("still no"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::find_for_component_id(ctx, component_jack.id()).await?;
    assert!(!action_ids.is_empty());
    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;
        assert_ne!(
            ActionKind::Update,
            Action::prototype(ctx, action.id()).await?.kind
        );
    }

    // ======================================================
    // Updating values in a component that has a resource should enqueue an update action
    // ======================================================
    Action::remove_all_for_component_id(ctx, component_jack.id()).await?;

    // Note: we're updating the root/si/name - which propagates to root/domain/name
    // and as this component has a resource, DVU should be enqueuing the update func!
    AttributeValue::update(ctx, av_id, Some(serde_json::json!("whomever"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let mut action_ids = Action::find_for_component_id(ctx, component_jack.id()).await?;
    assert_eq!(1, action_ids.len());
    let action_id = action_ids.pop().expect("no actions found for jack");
    let action = Action::get_by_id(ctx, action_id).await?;
    assert_eq!(
        ActionKind::Update,
        Action::prototype(ctx, action.id()).await?.kind
    );
    assert_eq!(ActionState::Queued, action.state());

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

    Action::remove_all_for_component_id(ctx, component_swift.id()).await?;
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

    Frame::upsert_parent_for_tests(ctx, second_component.id(), first_component.id()).await?;
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
    Frame::upsert_parent_for_tests(ctx, second_component_id, first_component_id).await?;
    Frame::upsert_parent_for_tests(ctx, third_component_id, second_component_id).await?;

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
async fn refresh_actions_run_where_they_should(ctx: &mut DalContext) -> Result<()> {
    // small even schema can be used to test this!
    // First, we'll create a new component and apply (and wait for the create action to run)
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await?;
    let no_payload_yet = value::has_value(ctx, ("small even lego", "/resource")).await?;
    assert!(!no_payload_yet);
    let av_id = Component::attribute_value_for_prop(
        ctx,
        small_even_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await?;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("import id"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // should be only 1 action enqueued, the create action
    let actions = Action::list_topologically(ctx).await?;
    assert!(actions.len() == 1);

    // Apply change set to head
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    let actions = Action::list_topologically(ctx).await?;

    assert!(actions.is_empty());
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Refresh func has run once
    let payload = value::get(ctx, ("small even lego", "/resource/payload")).await?;
    let refresh_count = payload
        .get("refresh_count")
        .and_then(|v| v.as_u64())
        .expect("has a refresh_count");
    assert_eq!(serde_json::json!(1), refresh_count);

    // then we'll explicity run refresh, and see that it runs on head as expected
    Action::enqueue_refresh_in_correct_change_set_and_commit(ctx, small_even_lego.id()).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let actions = Action::list_topologically(ctx).await?;
    assert!(actions.len() == 1);
    let solo_action = actions.first().expect("confirmed we have 1");
    let action = Action::get_by_id(ctx, *solo_action).await?;
    // the action that was enqueued, was enqueued on head, which is our current change set
    assert_eq!(action.originating_changeset_id(), ctx.change_set_id());
    assert_eq!(
        ctx.change_set_id(),
        ctx.get_workspace_default_change_set_id().await?
    );

    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;
    // check that refresh ran again
    let payload = value::get(ctx, ("small even lego", "/resource/payload")).await?;
    let refresh_count = payload
        .get("refresh_count")
        .and_then(|v| v.as_u64())
        .expect("has a refresh_count");
    assert_eq!(serde_json::json!(2), refresh_count);

    // now let's fork head, and run refresh again, and see that it's enqueued on head
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    Action::enqueue_refresh_in_correct_change_set_and_commit(ctx, small_even_lego.id()).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let seconds = 10;
    let mut did_pass = false;
    for _ in 0..(seconds * 10) {
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let actions = Action::list_topologically(ctx).await?;

        if actions.len() == 1 {
            let solo_action = actions.first().expect("confirmed we have 1");
            let action = Action::get_by_id(ctx, *solo_action).await?;
            // action originated on head, which is not our current change set
            assert!(
                action.originating_changeset_id()
                    == ctx.get_workspace_default_change_set_id().await?
            );
            assert_ne!(
                ctx.get_workspace_default_change_set_id().await?,
                ctx.change_set_id()
            );
            did_pass = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if !did_pass {
        panic!("Should have seen an action enqueued on head, but did not. Must investigate!");
    }
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;
    let payload = value::get(ctx, ("small even lego", "/resource/payload")).await?;
    let refresh_count = payload
        .get("refresh_count")
        .and_then(|v| v.as_u64())
        .expect("has a refresh_count");
    assert_eq!(serde_json::json!(3), refresh_count);
    Ok(())
}

#[test]
async fn resource_value_propagation_subscriptions_works(ctx: &mut DalContext) -> Result<()> {
    // create 2 variants A & B where A has a resource_value prop that B is subscribed to
    let component_a_code_definition = r#"
        function main() {
            const prop = new PropBuilder()
                .setName("prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();
            const resourceProp = new PropBuilder()
                .setName("prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();

            return new AssetBuilder()
                .addProp(prop)
                .addResourceProp(resourceProp)
                .build();
        }
    "#;

    let a_variant = ExpectSchemaVariant(
        VariantAuthoringClient::create_schema_and_variant_from_code(
            ctx,
            "A",
            None,
            None,
            "Category",
            "#0077cc",
            component_a_code_definition,
        )
        .await?
        .id,
    );

    // Create Action Func for A
    let func_name = "Create A".to_string();
    let func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Create,
        a_variant.id(),
    )
    .await?;

    let create_func_code = r#"
        async function main(component: Input): Promise<Output> {
        const prop = component.properties.domain?.prop;
    return {
        status: "ok",
        payload: {
            prop: prop
        },
    }
}
    "#;
    FuncAuthoringClient::save_code(ctx, func.id, create_func_code).await?;

    // Create B Variant
    let component_b_code_definition = r#"
        function main() {
            const prop = new PropBuilder()
                .setName("prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();
            return new AssetBuilder()
                .addProp(prop)
                .build();
        }
    "#;

    let _b_variant = ExpectSchemaVariant(
        VariantAuthoringClient::create_schema_and_variant_from_code(
            ctx,
            "B",
            None,
            None,
            "Category",
            "#0077cc",
            component_b_code_definition,
        )
        .await?
        .id,
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let all_funcs = SchemaVariant::all_funcs(ctx, a_variant.id())
        .await
        .expect("unable to get all funcs");

    // do we see resourcePayloadToValue??
    assert!(
        all_funcs
            .iter()
            .any(|func| func.name == "si:resourcePayloadToValue")
    );

    // create each component
    component::create(ctx, "A", "A").await?;
    component::create(ctx, "B", "B").await?;

    // component b subscribes to resource_value/prop from component a
    value::subscribe(ctx, ("B", "/domain/prop"), ("A", "/resource_value/prop")).await?;

    // update the value for component A
    value::set(ctx, ("A", "/domain/prop"), "hello world").await?;
    // commit change set
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let actions = Action::list_topologically(ctx).await?;
    assert!(actions.len() == 1);

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;
    // also wait for dvu!
    ChangeSet::wait_for_dvu(ctx, false).await?;
    // need to update snapshot to visibility again for some reason??
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert!(
        value::has_value(ctx, ("A", "/resource/payload"))
            .await
            .expect("Failed to get value")
    );

    assert_eq!(
        json!("hello world"),
        value::get(ctx, ("A", "/resource_value/prop")).await?
    );
    // check if value has propagated
    //  assert!(value::has_value(ctx,  ("B", "/domain/prop")).await?);
    assert_eq!(
        json!("hello world"),
        value::get(ctx, ("B", "/domain/prop")).await?
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
    value::subscribe(ctx, (b, "/domain/two"), (a, "/domain/two")).await?;
    assert_eq!(vec!["Create a", "Create c"], next_actions(ctx).await?);
    value::subscribe(ctx, (c, "/domain/one"), (b, "/domain/one")).await?;
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
    value::subscribe(ctx, (c, "/domain/one"), (b, "/domain/one")).await?;
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
    value::subscribe(ctx, (b, "/domain/two"), (a, "/domain/two")).await?;
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
    value::subscribe(ctx, (b, "/domain/two"), (a, "/domain/two")).await?;
    assert_eq!(vec!["Destroy b", "Destroy c"], next_actions(ctx).await?);
    value::subscribe(ctx, (c, "/domain/one"), (b, "/domain/one")).await?;
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
    value::subscribe(ctx, (c, "/domain/one"), (b, "/domain/one")).await?;
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
    value::subscribe(ctx, (b, "/domain/two"), (a, "/domain/two")).await?;
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

#[test]
async fn refresh_action_doesnt_dispatch_without_resource_in_change_set(
    ctx: &mut DalContext,
) -> Result<()> {
    // Create a component without setting up a resource
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "component without resource",
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Create action should be enqueued
    let action_ids = Action::find_for_component_id(ctx, component.id()).await?;
    let action_id = action_ids.first().expect("has an action");
    let action = Action::get_by_id(ctx, *action_id).await?;
    assert_eq!(action.state(), ActionState::Queued);

    // Try and run refresh in this change set
    Action::enqueue_refresh_in_correct_change_set_and_commit(ctx, component.id()).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now there's also a refresh action enqueued, NOT DISPATCHED!
    let action_ids =
        Action::find_for_kind_and_component_id(ctx, component.id(), ActionKind::Refresh).await?;
    let action_id = action_ids.first().expect("has one");
    let action = Action::get_by_id(ctx, *action_id).await?;
    assert_eq!(action.state(), ActionState::Queued);

    // now try again and this should no-op
    let result =
        Action::enqueue_refresh_in_correct_change_set_and_commit(ctx, component.id()).await;
    assert!(result.is_ok());

    Ok(())
}

#[test]
async fn refresh_action_failed_requeue_on_head(ctx: &mut DalContext) -> Result<()> {
    // Create a component with refresh functionality (using small even lego)
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "test component",
    )
    .await?;

    // Set up the component with a resource ID so it can have a refresh action
    let av_id =
        Component::attribute_value_for_prop(ctx, component.id(), &["root", "si", "resourceId"])
            .await?;
    AttributeValue::update(ctx, av_id, Some(serde_json::json!("test-resource-id"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Apply change set to head and wait for create action to run
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    // Verify we're on head
    assert_eq!(
        ctx.change_set_id(),
        ctx.get_workspace_default_change_set_id().await?
    );

    // Enqueue a refresh action on head
    // create a new refresh action manually and set it to failed to simulate a failed action
    let refresh_action =
        ActionPrototype::for_variant(ctx, component.schema_variant(ctx).await?.id()).await?;
    for action_proto in refresh_action {
        if action_proto.kind == ActionKind::Refresh {
            let refresh = Action::new(ctx, action_proto.id(), Some(component.id())).await?;
            Action::set_state(ctx, refresh.id(), ActionState::Failed).await?;
        }
    }
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // Find the refresh action and set it to Failed state
    let refresh_actions = Action::find_for_component_id(ctx, component.id()).await?;
    let mut refresh_action_id = None;
    for action_id in refresh_actions {
        let prototype = Action::prototype(ctx, action_id).await?;
        if prototype.kind == ActionKind::Refresh {
            refresh_action_id = Some(action_id);
            break;
        }
    }
    let refresh_action_id = refresh_action_id.expect("Should have a refresh action");

    // Verify the action is now in Failed state
    let failed_action = Action::get_by_id(ctx, refresh_action_id).await?;
    assert_eq!(failed_action.state(), ActionState::Failed);

    // Now call enqueue_refresh_in_correct_change_set_and_commit again
    // This should requeue the failed action back to Queued state since we're on head
    Action::enqueue_refresh_in_correct_change_set_and_commit(ctx, component.id()).await?;

    // Verify that there's now a refresh action in Queued state
    let actions_after_requeue = Action::find_for_component_id(ctx, component.id()).await?;
    let mut queued_refresh_action = None;
    for action_id in actions_after_requeue {
        let action = Action::get_by_id(ctx, action_id).await?;
        let prototype = Action::prototype(ctx, action_id).await?;
        if prototype.kind == ActionKind::Refresh && action.state() == ActionState::Queued {
            queued_refresh_action = Some(action_id);
            break;
        }
    }
    let queued_refresh_action =
        queued_refresh_action.expect("Should have a queued refresh action after requeue");

    // Verify the action is in Queued state
    let requeued_action = Action::get_by_id(ctx, queued_refresh_action).await?;
    assert_eq!(requeued_action.state(), ActionState::Queued);

    // Verify we're still on head
    assert_eq!(
        ctx.change_set_id(),
        ctx.get_workspace_default_change_set_id().await?
    );

    Ok(())
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
