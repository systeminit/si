use dal::{
    action::prototype::ActionKind, action::prototype::ActionPrototype, action::Action,
    action::ActionState, func::binding::FuncBinding, func::execution::FuncExecution,
    func::intrinsics::IntrinsicFunc, AttributeValue, Component, DalContext, Func,
};
use dal_test::helpers::create_component_for_schema_name;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn prototype_id(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
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
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
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
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
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
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
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
async fn set_func_execution(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let prototypes = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant");
    assert!(!prototypes.is_empty());
    for prototype in prototypes {
        if prototype.kind == ActionKind::Create {
            let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
                .await
                .expect("unable to find identity func");
            let identity_func = Func::get_by_id(ctx, identity_func_id)
                .await
                .expect("unable to get func by id")
                .expect("no func found for identity");
            let identity_func_binding = FuncBinding::new(
                ctx,
                serde_json::Value::Null,
                identity_func.id,
                identity_func.backend_kind,
            )
            .await
            .expect("unable to create func_binding");
            let func_execution = FuncExecution::new(ctx, &identity_func, &identity_func_binding)
                .await
                .expect("unable to create func execution");

            let action = Action::new(ctx, prototype.id, Some(component.id()))
                .await
                .expect("unable to upsert action");
            assert_eq!(
                action
                    .func_execution(ctx)
                    .await
                    .expect("unable to find func execution"),
                None
            );

            Action::set_func_execution_pk(ctx, action.id(), Some(func_execution.pk()))
                .await
                .expect("unable to set func execution pk");

            let action = Action::get_by_id(ctx, action.id())
                .await
                .expect("unable to get action by id");
            assert_eq!(
                action
                    .func_execution(ctx)
                    .await
                    .expect("unable to find func execution"),
                Some(func_execution)
            );
            break;
        }
    }
}

#[test]
async fn run(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "swifty", "shake it off").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");
    let proto = ActionPrototype::for_variant(ctx, variant_id)
        .await
        .expect("unable to list prototypes for variant")
        .pop()
        .expect("unable to find prototype for variant");
    let action = Action::new(ctx, proto.id, Some(component.id()))
        .await
        .expect("unable to create action runner");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(Action::run(ctx, action.id())
        .await
        .expect("unable to run")
        .is_some());
}

#[test]
async fn auto_queue_creation(ctx: &mut DalContext) {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component = create_component_for_schema_name(ctx, "swifty", "jack antonoff").await;
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
    let component = create_component_for_schema_name(ctx, "swifty", "jack antonoff").await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("commit and update snapshot to visibility");

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx, true)
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
