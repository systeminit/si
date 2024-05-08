use dal::{
    action::prototype::ActionKind, action::prototype::ActionPrototype, action::Action,
    action::ActionState, func::binding::FuncBinding, func::execution::FuncExecution,
    func::intrinsics::IntrinsicFunc, Component, DalContext, Func,
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
