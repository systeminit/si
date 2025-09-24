use std::time::Duration;

use dal::{
    AttributeValue,
    ChangeSet,
    Component,
    ComponentId,
    DalContext,
    Func,
    SchemaVariantId,
    Ulid,
    action::Action,
    component::resource::ResourceData,
    diagram::{
        geometry::Geometry,
        view::View,
    },
    management::{
        ManagementFuncReturn,
        ManagementGeometry,
        ManagementOperator,
        prototype::{
            ManagementFuncKind,
            ManagementPrototype,
            ManagementPrototypeError,
            ManagementPrototypeExecution,
        },
    },
};
use dal_test::{
    Report,
    Result,
    expected::{
        ExpectComponent,
        ExpectSchemaVariant,
        ExpectView,
    },
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        change_set,
        component::{
            self,
            find_management_prototype,
        },
        create_component_for_default_schema_name_in_default_view,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use si_db::{
    ManagementFuncJobState,
    ManagementState,
};
use si_events::ActionState;
use si_frontend_types::RawGeometry;
use si_id::ViewId;
use tokio::try_join;
use veritech_client::{
    ManagementFuncStatus,
    ResourceStatus,
};

async fn exec_mgmt_func(
    ctx: &DalContext,
    component_id: ComponentId,
    prototype_name: &str,
    view_id: Option<ViewId>,
) -> Result<(ManagementPrototypeExecution, ManagementFuncReturn)> {
    let management_prototype = find_management_prototype(ctx, component_id, prototype_name).await?;

    let mut execution_result = management_prototype
        .execute(ctx, component_id, view_id)
        .await
        .map_err(|err| {
            if let ManagementPrototypeError::FuncExecutionFailure(ref err) = err {
                println!("Error: {err}");
            }
            err
        })?;

    let result: ManagementFuncReturn = execution_result
        .result
        .take()
        .expect("should have a result success")
        .try_into()?;

    Ok((execution_result, result))
}

#[test]
async fn set_resource(ctx: &mut DalContext) -> Result<()> {
    // Create a schema that has mirrored resource_value + domain props
    let _test_variant_id = variant::create(
        ctx,
        "test::resource",
        r#"
            function main() {
                return {
                    props: [
                        {
                            name: "name",
                            kind: "string"
                        },
                        {
                            name: "value",
                            kind: "string"
                        }
                    ],
                    resourceProps: [
                        {
                            name: "name", 
                            kind: "string"
                        },
                        {
                            name: "value",
                            kind: "string"
                        }
                    ]
                };
            }
        "#,
    )
    .await?;

    // Write a management function that sets both domain and resource
    let _func_id = variant::create_management_func(
        ctx,
        "test::resource",
        "Set Resource",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
            const result = {
                "name": "test-resource",
                "value": "resource-value",
                };
                return {
                    status: "ok",
                    ops: {
                        update: {
                            self: {
                                attributes: {
                                   "/domain" : result,
                                    "/resource": result,
                                }
                            }
                        }
                    }
                };
            }
        "##,
    )
    .await?;

    // Create a component for the new schema variant
    let test_component = component::create(ctx, "test::resource", "test-component").await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Find and enqueue the management job
    let management_prototype =
        find_management_prototype(ctx, test_component, "Set Resource").await?;

    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        management_prototype.id(),
        test_component,
        None,
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    ChangeSetTestHelpers::wait_for_mgmt_job_to_run(ctx, management_prototype.id(), test_component)
        .await?;
    ChangeSet::wait_for_dvu(ctx, false).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Ensure that resource_value gets propagated
    // Check domain values were set
    let domain_name = value::get(ctx, (test_component, "/domain/name")).await?;
    let domain_value = value::get(ctx, (test_component, "/domain/value")).await?;
    let diff = Component::get_diff(ctx, test_component).await?;
    dbg!(&diff.diff);
    assert_eq!(serde_json::json!("test-resource"), domain_name);
    assert_eq!(serde_json::json!("resource-value"), domain_value);

    // Check resource_value props were set
    let resource_name = value::get(ctx, (test_component, "/resource_value/name")).await?;
    let resource_value = value::get(ctx, (test_component, "/resource_value/value")).await?;

    assert_eq!(serde_json::json!("test-resource"), resource_name);
    assert_eq!(serde_json::json!("resource-value"), resource_value);

    Ok(())
}

#[test]
async fn import_and_refresh(ctx: &mut DalContext) -> Result<()> {
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
    let management_prototype =
        find_management_prototype(ctx, small_even_lego.id(), "Import from AWS").await?;
    assert!(
        ManagementPrototype::kind_by_id(ctx, management_prototype.id()).await?
            == ManagementFuncKind::Import
    );

    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        management_prototype.id(),
        small_even_lego.id(),
        None,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let actions = Action::all_ids(ctx).await?;
    // there should be 1 refresh action now
    assert!(actions.len() == 1);
    let av_id =
        Component::attribute_value_for_prop(ctx, small_even_lego.id(), &["root", "domain", "one"])
            .await?;

    let two_av = AttributeValue::get_by_id(ctx, av_id).await?;

    let two_value = two_av.value(ctx).await?;

    assert_eq!(Some(serde_json::json!("twostep")), two_value);

    let seconds = 10;
    let mut did_pass = false;
    for _ in 0..(seconds * 10) {
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let actions = Action::list_topologically(ctx).await?;

        if actions.is_empty() {
            did_pass = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if !did_pass {
        panic!(
            "Refresh action should have been dispatched in this change set, but it did not. Must investigate!"
        );
    }
    // resource has been set!
    let payload = value::get(ctx, ("small even lego", "/resource/payload")).await?;
    let refresh_count = payload
        .get("refresh_count")
        .and_then(|v| v.as_u64())
        .expect("has a refresh_count");
    assert_eq!(serde_json::json!(1), refresh_count);

    // now let's run refresh manually and see that we're refreshed in this change set!
    Action::enqueue_refresh_in_correct_change_set_and_commit(ctx, small_even_lego.id()).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    let payload = value::get(ctx, ("small even lego", "/resource/payload")).await?;
    let refresh_count = payload
        .get("refresh_count")
        .and_then(|v| v.as_u64())
        .expect("has a refresh_count");
    assert_eq!(serde_json::json!(2), refresh_count);

    // now apply the change set
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    // fork head, and run import again. Refresh should not run in this case.
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        management_prototype.id(),
        small_even_lego.id(),
        None,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // check actions, there should be one refresh enqueued but not dispatched since this component now exists on head
    let actions = Action::all_ids(ctx).await?;
    assert!(actions.len() == 1);
    let action_id = actions.first().expect("has one");
    let action = Action::get_by_id(ctx, *action_id).await?;
    assert_eq!(action.state(), ActionState::Queued);

    Ok(())
}

#[test]
async fn update_managed_components_in_view(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await?;

    let view_name = "a view askew";
    let new_view_id = ExpectView::create_with_name(ctx, view_name).await.id();
    Geometry::new_for_component(ctx, small_odd_lego.id(), new_view_id).await?;
    Geometry::new_for_component(ctx, small_even_lego.id(), new_view_id).await?;

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id()).await?;

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Update in View",
        Some(new_view_id),
    )
    .await?;

    assert_eq!(ManagementFuncStatus::Ok, result.status);
    assert_eq!(Some(view_name), result.message.as_deref());

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        Some(new_view_id),
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    let mut new_component = None;
    let components = Component::list(ctx).await?;
    assert_eq!(2, components.len());
    for c in components {
        if c.name(ctx).await? == "small even lego managed by small odd lego" {
            new_component = Some(c);
            break;
        }
    }

    let new_component = new_component.expect("should have found the cloned component");
    let default_view_id = View::get_id_for_default(ctx).await?;
    let default_view_geometry = new_component.geometry(ctx, default_view_id).await?;

    assert_eq!(0, default_view_geometry.x());
    assert_eq!(0, default_view_geometry.y());

    let new_view_geometry = new_component.geometry(ctx, new_view_id).await?;

    assert_eq!(1000, new_view_geometry.x());
    assert_eq!(750, new_view_geometry.y());
    Ok(())
}

#[test]
async fn update_managed_components(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await?;

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id()).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Update", None).await?;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    let mut new_component = None;
    let components = Component::list(ctx).await?;
    assert_eq!(2, components.len());
    for c in components {
        if c.name(ctx).await? == "small even lego managed by small odd lego" {
            new_component = Some(c);
            break;
        }
    }

    let _new_component = new_component.expect("should have found the cloned component");

    Ok(())
}

#[test]
async fn create_component_of_other_schema(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await?;

    let av_id = Component::attribute_value_for_prop(
        ctx,
        small_even_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await?;

    AttributeValue::update(
        ctx,
        av_id,
        Some(serde_json::json!("small even lego resource id")),
    )
    .await?;

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id()).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Clone", None).await?;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    let mut new_component = None;
    let components = Component::list(ctx).await?;
    assert_eq!(4, components.len());
    for c in components {
        if c.name(ctx).await? == "small even lego_clone" {
            new_component = Some(c);
            break;
        }
    }

    let new_component = new_component.expect("should have found the cloned component");
    let av_id =
        Component::attribute_value_for_prop(ctx, new_component.id(), &["root", "si", "resourceId"])
            .await?;

    let av = AttributeValue::get_by_id(ctx, av_id).await?;

    assert_eq!(
        Some(json!("small even lego resource id")),
        av.value(ctx).await?
    );

    Ok(())
}

#[test]
async fn create_component_of_same_schema(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Clone", None).await?;
    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    let mut new_component = None;
    let components = Component::list(ctx).await?;
    assert_eq!(2, components.len());
    for c in components {
        if c.name(ctx).await? == "small odd lego_clone" {
            new_component = Some(c);
            break;
        }
    }

    let default_view_id = View::get_id_for_default(ctx).await?;

    let new_component = new_component.expect("should have found the cloned component");
    let new_geometry: ManagementGeometry = new_component
        .geometry(ctx, default_view_id)
        .await?
        .into_raw()
        .into();

    assert_eq!(Some(10.0), new_geometry.x);
    assert_eq!(Some(20.0), new_geometry.y);

    let managers = new_component.managers(ctx).await?;

    assert_eq!(1, managers.len());
    assert_eq!(
        small_odd_lego.id(),
        managers[0],
        "should have the same manager"
    );

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Clone", None).await?;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    let mut new_component_2 = None;

    let components = Component::list(ctx).await?;
    assert_eq!(4, components.len());
    for c in components {
        let name = c.name(ctx).await?;
        if name == "small odd lego_clone_clone" {
            new_component_2 = Some(c);
            //break;
        }
    }
    let _new_component_2 = new_component_2.expect("should have found the cloned component again");

    Ok(())
}

#[test]
async fn execute_management_func(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    let av_id = Component::attribute_value_for_prop(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await?;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("import id"))).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Import small odd lego", None).await?;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    let av_id =
        Component::attribute_value_for_prop(ctx, small_odd_lego.id(), &["root", "domain", "two"])
            .await?;

    let two_av = AttributeValue::get_by_id(ctx, av_id).await?;

    let two_value = two_av.value(ctx).await?;

    assert_eq!(Some(serde_json::json!("step")), two_value);

    Ok(())
}

#[test]
async fn create_in_views(ctx: &DalContext) -> Result<()> {
    let mut small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    let default_view_id = View::get_id_for_default(ctx).await?;

    let manager_x = 50;
    let manager_y = 75;

    small_odd_lego
        .set_geometry(ctx, default_view_id, manager_x, manager_y, None, None)
        .await?;

    let av_id = Component::attribute_value_for_prop(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await?;

    let view_name = "the black lodge";
    let view = ExpectView::create_with_name(ctx, view_name).await;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(view_name))).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Create in Other Views", None).await?;

    let operations = result.operations.expect("should have operations");

    let component_id = ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?
    .expect("should return component ids")
    .pop()
    .expect("should have a component id");

    let geometries = Geometry::by_view_for_component_id(ctx, component_id).await?;

    assert_eq!(2, geometries.len());

    let black_lodge_geometry = geometries
        .get(&view.id())
        .cloned()
        .expect("has a geometry in the black lodge");

    assert_eq!(15, black_lodge_geometry.x() - manager_x);
    assert_eq!(15, black_lodge_geometry.y() - manager_y);

    let default_geometry = geometries
        .get(&default_view_id)
        .cloned()
        .expect("has a geometry in the default view");

    assert_eq!(100, default_geometry.x() - manager_x);
    assert_eq!(100, default_geometry.y() - manager_y);

    Ok(())
}

#[test]
async fn create_view_and_in_view(ctx: &DalContext) -> Result<()> {
    let mut small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    let default_view_id = View::get_id_for_default(ctx).await?;

    let manager_x = 50;
    let manager_y = 75;

    small_odd_lego
        .set_geometry(ctx, default_view_id, manager_x, manager_y, None, None)
        .await?;

    let av_id = Component::attribute_value_for_prop(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await?;

    let view_name = "the red room";
    AttributeValue::update(ctx, av_id, Some(serde_json::json!(view_name))).await?;

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Create View and Component in View",
        None,
    )
    .await?;

    let operations = result.operations.expect("should have operations");

    let component_id = ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?
    .expect("should return component ids")
    .pop()
    .expect("should have a component id");

    let red_room = View::find_by_name(ctx, view_name)
        .await?
        .expect("find view");

    let geometries = Geometry::by_view_for_component_id(ctx, component_id).await?;

    assert_eq!(1, geometries.len());

    let red_room_geo = geometries
        .get(&red_room.id())
        .cloned()
        .expect("has a geometry in the red room");

    assert_eq!(315, red_room_geo.x() - manager_x);
    assert_eq!(315, red_room_geo.y() - manager_y);

    Ok(())
}

#[test]
async fn delete_and_erase_components(ctx: &mut DalContext) -> Result<()> {
    let manager = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "middle manager",
    )
    .await?;

    let component_with_resource_to_delete =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to delete",
        )
        .await?;

    let component_still_on_head = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component still on head",
    )
    .await?;

    let component_with_resource_to_erase =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to erase",
        )
        .await?;

    for component_id in [
        component_with_resource_to_delete.id(),
        component_still_on_head.id(),
        component_with_resource_to_erase.id(),
    ] {
        Component::manage_component(ctx, manager.id(), component_id).await?;
    }

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );

    component_with_resource_to_delete
        .set_resource(ctx, resource_data.clone())
        .await?;
    component_with_resource_to_erase
        .set_resource(ctx, resource_data.clone())
        .await?;

    let component_to_delete = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component to delete",
    )
    .await?;

    Component::manage_component(ctx, manager.id(), component_to_delete.id()).await?;

    let av_id =
        Component::attribute_value_for_prop(ctx, manager.id(), &["root", "si", "resourceId"])
            .await?;

    let resource_id = format!(
        "{},{},{},{}",
        component_to_delete.name(ctx).await?,
        component_with_resource_to_delete.name(ctx).await?,
        component_still_on_head.name(ctx).await?,
        component_with_resource_to_erase.name(ctx).await?
    );

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(resource_id))).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, manager.id(), "Delete and Erase", None).await?;
    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        manager.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    assert!(
        Component::try_get_by_id(ctx, component_to_delete.id())
            .await?
            .is_none(),
        "deleted component should be gone"
    );

    assert!(
        Component::try_get_by_id(ctx, component_still_on_head.id())
            .await?
            .is_none(),
        "deleted component that is still on head should be gone in this change set"
    );

    assert!(
        Component::exists_on_head_by_ids(ctx, &[component_still_on_head.id()])
            .await?
            .contains(&component_still_on_head.id()),
        "component should still exist on head"
    );

    assert!(
        Component::try_get_by_id(ctx, component_with_resource_to_erase.id())
            .await?
            .is_none(),
        "erased component should be gone"
    );

    let component_with_resource_to_delete =
        Component::get_by_id(ctx, component_with_resource_to_delete.id()).await?;
    assert!(
        component_with_resource_to_delete.to_delete(),
        "component with resource should be marked as to delete"
    );

    Ok(())
}

#[test]
async fn remove_view_and_component_from_view(ctx: &DalContext) -> Result<()> {
    let manager =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "c-suite")
            .await?;

    let new_view_name = "vista del mar";
    let new_view = ExpectView::create_with_name(ctx, new_view_name).await;

    let component_in_both_views_1 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "both views 1",
    )
    .await?;

    let component_in_both_views_2 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "both views 2",
    )
    .await?;

    for component_id in [
        component_in_both_views_1.id(),
        component_in_both_views_2.id(),
    ] {
        let raw_geometry = RawGeometry {
            x: 25,
            y: 25,
            width: None,
            height: None,
        };

        Component::manage_component(ctx, manager.id(), component_id).await?;

        Component::add_to_view(ctx, component_id, new_view.id(), raw_geometry).await?;
    }

    let av_id =
        Component::attribute_value_for_prop(ctx, manager.id(), &["root", "si", "resourceId"])
            .await?;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(new_view_name))).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, manager.id(), "Remove View and Components", None).await?;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        manager.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    assert!(View::find_by_name(ctx, new_view_name).await?.is_none());

    for component_id in [
        component_in_both_views_1.id(),
        component_in_both_views_2.id(),
    ] {
        let geometries = Geometry::by_view_for_component_id(ctx, component_id).await?;
        assert!(!geometries.is_empty());
        assert!(!geometries.contains_key(&new_view.id()));
    }

    // try to delete the default view
    let av_id =
        Component::attribute_value_for_prop(ctx, manager.id(), &["root", "si", "resourceId"])
            .await?;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("DEFAULT"))).await?;

    let default_view_id = View::get_id_for_default(ctx).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, manager.id(), "Remove View and Components", None).await?;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        manager.id(),
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

    assert_eq!(default_view_id, View::get_id_for_default(ctx).await?);

    for component_id in [
        component_in_both_views_1.id(),
        component_in_both_views_2.id(),
    ] {
        let geometries = Geometry::by_view_for_component_id(ctx, component_id).await?;
        assert!(!geometries.is_empty());
    }

    Ok(())
}

#[test]
async fn upgrade_manager_variant(ctx: &mut DalContext) -> Result<()> {
    // Set up management schema
    let original_variant = ExpectSchemaVariant(
        variant::create(
            ctx,
            "createme",
            r#"
            function main() {
                return new AssetBuilder().build();
            }
        "#,
        )
        .await?,
    );
    change_set::commit(ctx).await?;
    variant::create_management_func(
        ctx,
        "createme",
        "runme",
        r#"
                function main(input) {
                    return {
                        status: "ok",
                        ops: {
                            create: {
                                created: { kind: "createme" }
                            }
                        }
                    }
                }
            "#,
    )
    .await?;
    change_set::commit(ctx).await?;

    // Create manager component and run management function to create managed component
    let manager = original_variant.create_component_on_default_view(ctx).await;
    component::execute_management_func(ctx, manager.id(), "runme").await?;
    let created = ExpectComponent::find(ctx, "created").await;
    assert_eq!(created.schema_variant(ctx).await, original_variant);

    // Check that the managed component is in the list
    assert_eq!(
        manager.component(ctx).await.get_managed(ctx).await?,
        vec![created.id()]
    );

    // Regenerate the schema variant and ensure both components got upgraded
    change_set::commit(ctx).await?;
    let new_variant = original_variant.regenerate(ctx).await;
    assert_ne!(new_variant, original_variant);
    change_set::commit(ctx).await?;
    assert_eq!(manager.schema_variant(ctx).await, new_variant);
    assert_eq!(created.schema_variant(ctx).await, new_variant);

    // Check that the managed component is still in the list
    assert_eq!(
        manager.component(ctx).await.get_managed(ctx).await?,
        vec![created.id()]
    );

    Ok(())
}

#[test]
async fn create_and_subscribe_to_source(ctx: &mut DalContext) -> Result<()> {
    create_subscription_tracker_asset(ctx).await?;

    // Create management function to subscribe to the source component
    variant::create_management_func(
        ctx,
        "subscription_tracker",
        "create_and_subscribe_to_source",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
                return {
                    status: "ok",
                    ops: {
                        create: {
                            NewComponent: {
                                attributes: {
                                    "/domain/Value": { $source: { component: "source", path: "/domain/Value" } },
                                }
                            }
                        },
                        update: {
                            self: {
                                attributes: {
                                    "/domain/Value": { $source: { component: "NewComponent", path: "/domain/Value" } },
                                }
                            }
                        }
                    }
                };
            }
        "##,
    )
    .await?;

    // Create source and manager manager components
    component::create(ctx, "subscription_tracker", "source").await?;
    value::set(ctx, ("source", "/domain/Value"), "value from source").await?;
    component::create(ctx, "subscription_tracker", "manager").await?;
    change_set::commit(ctx).await?;

    // Use management function to subscribe source -> NewComponent -> manager
    component::execute_management_func(ctx, "manager", "create_and_subscribe_to_source").await?;
    change_set::commit(ctx).await?;
    component::execute_management_func(ctx, "manager", "save_subscriptions").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Value": "value from source"
        }),
        component::domain(ctx, "source").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from source",
            "Sources": serde_json::to_string_pretty(&json!({
                "/domain/Value": { "component": component::id(ctx, "NewComponent").await?, "path": "/domain/Value" }
            }))?,
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from source",
            "Sources": serde_json::to_string_pretty(&json!({
                "/domain/Value": { "component": component::id(ctx, "source").await?, "path": "/domain/Value" }
            }))?
        }),
        component::domain(ctx, "NewComponent").await?,
    );

    Ok(())
}

// create_and_subscribe_foo_to_bar, create_and_subscribe_bar_to_foo, and create_and_subscribe_foo_to_bar_reverse
// are designed to make sure we can create two components and subscribe one to the other,
// completely independent of of creation order:
// - If foo_to_bar and foo_to_bar_reverse both succeed, we are independent of the order specified in JS
// - If foo_to_bar and bar_to_foo both succeed, we are independent of the hash ordering (i.e.
//   if we pull it into a HashMap<name, Component>, Foo and Bar would always appear in the same
//   order)

#[test]
async fn create_and_subscribe_foo_to_bar(ctx: &mut DalContext) -> Result<()> {
    create_subscription_tracker_asset(ctx).await?;

    // Management function to create and subscribe Foo -> Bar
    variant::create_management_func(
        ctx,
        "subscription_tracker",
        "create_and_subscribe_foo_to_bar",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
                return {
                    status: "ok",
                    ops: {
                        create: {
                            Bar: {
                                attributes: {
                                    "/domain/Value": "value from Bar",
                                }
                            },
                            Foo: {
                                attributes: {
                                    "/domain/Value": { $source: { component: "Bar", path: "/domain/Value" } },
                                }
                            }
                        }
                    }
                };
            }
        "##,
    )
    .await?;

    // Create manager component
    component::create(ctx, "subscription_tracker", "manager").await?;
    change_set::commit(ctx).await?;

    // Use management function to create and subscribe Foo -> Bar
    component::execute_management_func(ctx, "manager", "create_and_subscribe_foo_to_bar").await?;
    component::execute_management_func(ctx, "manager", "save_subscriptions").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Sources": "{}"
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": serde_json::to_string_pretty(&json!({
                "/domain/Value": { "component": component::id(ctx, "Bar").await?, "path": "/domain/Value" }
            }))?
        }),
        component::domain(ctx, "Foo").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": "{}"
        }),
        component::domain(ctx, "Bar").await?,
    );

    Ok(())
}

#[test]
async fn create_and_subscribe_bar_to_foo(ctx: &mut DalContext) -> Result<()> {
    create_subscription_tracker_asset(ctx).await?;

    // Management function to create and subscribe Bar -> Foo
    variant::create_management_func(
        ctx,
        "subscription_tracker",
        "create_and_subscribe_bar_to_foo",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
                return {
                    status: "ok",
                    ops: {
                        create: {
                            Foo: {
                                attributes: {
                                    "/domain/Value": "value from Foo",
                                }
                            },
                            Bar: {
                                attributes: {
                                    "/domain/Value": { $source: { component: "Foo", path: "/domain/Value" } },
                                }
                            },
                        }
                    }
                };
            }
        "##,
    )
    .await?;

    // Create manager component
    component::create(ctx, "subscription_tracker", "manager").await?;
    change_set::commit(ctx).await?;

    // Use management function to create and subscribe Bar -> Foo
    component::execute_management_func(ctx, "manager", "create_and_subscribe_bar_to_foo").await?;
    component::execute_management_func(ctx, "manager", "save_subscriptions").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Sources": "{}"
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Foo",
            "Sources": "{}"
        }),
        component::domain(ctx, "Foo").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Foo",
            "Sources": serde_json::to_string_pretty(&json!({
                "/domain/Value": { "component": component::id(ctx, "Foo").await?, "path": "/domain/Value" }
            }))?
        }),
        component::domain(ctx, "Bar").await?,
    );

    Ok(())
}

#[test]
async fn create_and_subscribe_foo_to_bar_reverse(ctx: &mut DalContext) -> Result<()> {
    create_subscription_tracker_asset(ctx).await?;

    // Management function to create and subscribe Foo -> Bar
    variant::create_management_func(
        ctx,
        "subscription_tracker",
        "create_and_subscribe_foo_to_bar_reverse",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
                return {
                    status: "ok",
                    ops: {
                        create: {
                            Bar: {
                                attributes: {
                                    "/domain/Value": "value from Bar",
                                }
                            },
                            Foo: {
                                attributes: {
                                    "/domain/Value": { $source: { component: "Bar", path: "/domain/Value" } },
                                }
                            }
                        }
                    }
                };
            }
        "##,
    )
    .await?;

    // Create manager component
    component::create(ctx, "subscription_tracker", "manager").await?;
    change_set::commit(ctx).await?;

    // Use management function to create and subscribe Foo -> Bar
    component::execute_management_func(ctx, "manager", "create_and_subscribe_foo_to_bar_reverse")
        .await?;
    component::execute_management_func(ctx, "manager", "save_subscriptions").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({
            "Sources": "{}"
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": serde_json::to_string_pretty(&json!({
                "/domain/Value": { "component": component::id(ctx, "Bar").await?, "path": "/domain/Value" }
            }))?
        }),
        component::domain(ctx, "Foo").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": "{}"
        }),
        component::domain(ctx, "Bar").await?,
    );

    Ok(())
}

#[test]
async fn management_execution_state_db_test(ctx: &mut DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    change_set::commit(ctx).await?;

    let prototype_id = find_management_prototype(ctx, small_odd_lego.id(), "Clone")
        .await?
        .id();
    let prototype_2_id = find_management_prototype(ctx, small_odd_lego.id(), "Update")
        .await?
        .id();

    let ctx_clone = ctx.clone();
    let ctx_clone_2 = ctx.clone();
    let small_odd_lego_id = small_odd_lego.id();

    let write_1 = tokio::spawn(async move {
        let execution_state =
            ManagementFuncJobState::new_pending(&ctx_clone, small_odd_lego_id, prototype_id)
                .await?;
        ctx_clone.commit_no_rebase().await?;
        Ok::<_, Report>(execution_state)
    });

    let write_2 = tokio::spawn(async move {
        let execution_state =
            ManagementFuncJobState::new_pending(&ctx_clone_2, small_odd_lego_id, prototype_id)
                .await?;
        ctx_clone_2.commit_no_rebase().await?;
        Ok::<_, Report>(execution_state)
    });

    let result = try_join!(write_1, write_2).expect("should not have a join error");

    let execution = match result {
        (Ok(_), Ok(_)) => panic!("one should fail"),
        (Err(_), Err(_)) => panic!("one should fail, not both"),
        (_, Ok(result)) | (Ok(result), _) => result,
    };

    let ctx_clone = ctx.clone();
    ManagementFuncJobState::new_pending(&ctx_clone, small_odd_lego_id, prototype_id)
        .await
        .expect_err("should not allow new pending if one in pending state");

    let latest_execution =
        ManagementFuncJobState::get_latest_by_keys(&ctx_clone, small_odd_lego_id, prototype_id)
            .await?
            .expect("exists");

    assert_eq!(execution, latest_execution);

    let other_execution =
        ManagementFuncJobState::new_pending(&ctx_clone, small_odd_lego_id, prototype_2_id).await?;

    assert_ne!(execution.id(), other_execution.id());

    ManagementFuncJobState::transition_state(
        &ctx_clone,
        latest_execution.id(),
        ManagementState::Operating,
        None,
        None,
    )
    .await
    .expect_err("should not allow you to transition state");

    let new_state = ManagementFuncJobState::transition_state(
        &ctx_clone,
        latest_execution.id(),
        ManagementState::Executing,
        Some(Ulid::new().into()),
        None,
    )
    .await?;

    ctx_clone.commit_no_rebase().await?;
    let ctx_clone = ctx.clone();

    ManagementFuncJobState::new_pending(&ctx_clone, small_odd_lego_id, prototype_id)
        .await
        .expect_err("should not allow new pending if one in executing state");

    let latest_execution =
        ManagementFuncJobState::get_latest_by_keys(&ctx_clone, small_odd_lego_id, prototype_id)
            .await?
            .expect("exists");

    assert_eq!(new_state, latest_execution);
    assert_eq!(ManagementState::Executing, new_state.state());

    ManagementFuncJobState::transition_state(
        &ctx_clone,
        latest_execution.id(),
        ManagementState::Operating,
        Some(Ulid::new().into()),
        None,
    )
    .await?;

    ManagementFuncJobState::new_pending(&ctx_clone, small_odd_lego_id, prototype_id)
        .await
        .expect_err("should not allow new pending if one in operating state");

    let new_state = ManagementFuncJobState::transition_state(
        &ctx_clone,
        latest_execution.id(),
        ManagementState::Success,
        Some(Ulid::new().into()),
        None,
    )
    .await?;

    let latest_execution =
        ManagementFuncJobState::get_latest_by_keys(&ctx_clone, small_odd_lego_id, prototype_id)
            .await?
            .expect("exists");

    assert_eq!(new_state, latest_execution);
    assert_eq!(ManagementState::Success, new_state.state());

    let new_pending =
        ManagementFuncJobState::new_pending(&ctx_clone, small_odd_lego_id, prototype_id).await?;

    let pending = ManagementFuncJobState::get_pending(ctx, small_odd_lego_id, prototype_id)
        .await?
        .expect("pending should exist");

    assert_eq!(new_pending, pending);

    Ok(())
}

#[test]
async fn management_func_autosubscribe(ctx: &mut DalContext) -> Result<()> {
    // Create fake::region schema
    let _output = variant::create(
        ctx,
        "output",
        r#"
            function main() {
                return {
                    props: [
                        { name: "region", kind: "string", 
                            suggestAsSourceFor: [
                                    { schema: "fake::vpc", prop: "/domain/region" },
                                ]},
                    ],
                };
            }
        "#,
    )
    .await?;
    let _region_variant_id = variant::create(
        ctx,
        "fake::region",
        r#"
            function main() {
                return {
                    props: [
                        { name: "region", kind: "string", 
                            suggestAsSourceFor: [
                                    { schema: "fake::vpc", prop: "/domain/region" },
                                ]},
                    ],
                };
            }
        "#,
    )
    .await?;

    // Create fake::vpc schema
    let _vpc_variant_id = variant::create(
        ctx,
        "fake::vpc",
        r#"
            function main() {
                return {
                    props: [{
                                name: "region",
                                kind: "string",
                            },
                    ],
                    resourceProps: [
                         { 
                                name: "vpcId",
                                kind: "string",
                        
                            },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Create fake::subnet schema
    let _subnet_variant_id = variant::create(
        ctx,
        "fake::subnet",
        r#"
            function main() {
                return {
                    props: [
                      {
                                name: "region",
                                kind: "string",
                                suggestSources: [
                                    { schema: "fake::region", prop: "/domain/region" }
                                ]
                            },
                            { 
                                name: "vpcId",
                                kind: "string",
                                suggestSources: [
                                    { schema: "fake::vpc", prop: "/resource_value/vpcId" }
                                ]
                            }
                     
                    ],
                };
            }
        "#,
    )
    .await?;

    // Create discovery management function for VPC
    let func_id = variant::create_management_func(
        ctx,
        "fake::vpc",
        "Discover VPC",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
                return {
                    status: "ok",
                    ops: {
                        create: {
                            vpc1: {
                                kind: "fake::vpc",
                                attributes: {
                                    "/domain/region": "us-west-2",
                                    "/resource_value/vpcId": "vpc-12345"
                                }
                            },
                            vpc2: {
                                kind: "fake::vpc", 
                                attributes: {
                                    "/domain/region": "us-west-2",
                                    "/resource_value/vpcId": "vpc-67890"
                                }
                            }
                        }
                    }
                };
            }
        "##,
    )
    .await?;
    let mgmt = Func::get_by_id(ctx, func_id).await?;
    mgmt.modify(ctx, |func| {
        func.display_name = Some("Discover on AWS".to_owned());

        Ok(())
    })
    .await?;
    // Create discovery management function for Subnet
    let func_id = variant::create_management_func(
        ctx,
        "fake::subnet",
        "Discover Subnet",
        r##"
            async function main({ thisComponent }: Input): Promise<Output> {
                return {
                    status: "ok",
                    ops: {
                        create: {
                            subnet1: {
                                kind: "fake::subnet",
                                attributes: {
                                    "/domain/region": "us-west-2",
                                    "/domain/vpcId": "vpc-12345"
                                }
                            },
                            subnet2: {
                                kind: "fake::subnet",
                                attributes: {
                                    "/domain/region": "us-west-2",
                                    "/domain/vpcId": "vpc-67890"
                                }
                            }
                        }
                    }
                };
            }
        "##,
    )
    .await?;
    let mgmt = Func::get_by_id(ctx, func_id).await?;
    mgmt.modify(ctx, |func| {
        func.display_name = Some("Discover on AWS".to_owned());

        Ok(())
    })
    .await?;
    // Create region component
    let region = create_component_for_default_schema_name_in_default_view(
        ctx,
        "fake::region",
        "us-west-2-region",
    )
    .await?;

    let region_av_id =
        Component::attribute_value_for_prop(ctx, region.id(), &["root", "domain", "region"])
            .await?;
    AttributeValue::update(ctx, region_av_id, Some(serde_json::json!("us-west-2"))).await?;

    // Create VPC discovery manager
    let vpc_manager =
        create_component_for_default_schema_name_in_default_view(ctx, "fake::vpc", "vpc-manager")
            .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Find and run VPC discovery management function
    let vpc_management_prototype =
        find_management_prototype(ctx, vpc_manager.id(), "Discover VPC").await?;
    assert!(
        ManagementPrototype::kind_by_id(ctx, vpc_management_prototype.id()).await?
            == ManagementFuncKind::Discover
    );

    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        vpc_management_prototype.id(),
        vpc_manager.id(),
        None,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify VPC components were created and autosubscribed to region
    let components = Component::list(ctx).await?;
    let mut vpc1 = None;
    for c in &components {
        if c.name(ctx).await.unwrap() == "vpc1" {
            vpc1 = Some(c);
            break;
        }
    }
    let vpc1 = vpc1.expect("vpc1 component should exist");

    let vpc1_region_av_id =
        Component::attribute_value_for_prop(ctx, vpc1.id(), &["root", "domain", "region"]).await?;

    let vpc1_subscriptions = dal::AttributeValue::subscriptions(ctx, vpc1_region_av_id)
        .await?
        .expect("has subscriptions");
    assert_eq!(
        1,
        vpc1_subscriptions.len(),
        "vpc1 should have one subscription for region"
    );

    // Verify the subscription points to the region component
    let region_root_av_id = Component::root_attribute_value_id(ctx, region.id()).await?;
    assert_eq!(region_root_av_id, vpc1_subscriptions[0].attribute_value_id);
    assert_eq!("/domain/region", vpc1_subscriptions[0].path.to_string());

    // Create and run subnet discovery
    let subnet_manager = create_component_for_default_schema_name_in_default_view(
        ctx,
        "fake::subnet",
        "subnet-manager",
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let subnet_management_prototype =
        find_management_prototype(ctx, subnet_manager.id(), "Discover Subnet").await?;
    assert!(
        ManagementPrototype::kind_by_id(ctx, subnet_management_prototype.id()).await?
            == ManagementFuncKind::Discover
    );

    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        subnet_management_prototype.id(),
        subnet_manager.id(),
        None,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify subnet components were created and autosubscribed to both region and vpc
    let components = Component::list(ctx).await?;
    let mut subnet1 = None;
    for c in &components {
        if c.name(ctx).await.unwrap() == "subnet1" {
            subnet1 = Some(c);
            break;
        }
    }
    let subnet1 = subnet1.expect("subnet1 component should exist");

    // Check region subscription
    let subnet1_region_av_id =
        Component::attribute_value_for_prop(ctx, subnet1.id(), &["root", "domain", "region"])
            .await?;

    let subnet1_region_subscriptions =
        dal::AttributeValue::subscriptions(ctx, subnet1_region_av_id)
            .await?
            .expect("has subscriptions");
    assert_eq!(
        1,
        subnet1_region_subscriptions.len(),
        "subnet1 should have one subscription for region"
    );
    assert_eq!(
        region_root_av_id,
        subnet1_region_subscriptions[0].attribute_value_id
    );

    // Check VPC subscription
    let subnet1_vpc_av_id =
        Component::attribute_value_for_prop(ctx, subnet1.id(), &["root", "domain", "vpcId"])
            .await?;

    let subnet1_vpc_subscriptions = dal::AttributeValue::subscriptions(ctx, subnet1_vpc_av_id)
        .await?
        .expect("has subscriptions");
    assert_eq!(
        1,
        subnet1_vpc_subscriptions.len(),
        "subnet1 should have one subscription for vpcId"
    );

    let vpc1_root_av_id = Component::root_attribute_value_id(ctx, vpc1.id()).await?;
    assert_eq!(
        vpc1_root_av_id,
        subnet1_vpc_subscriptions[0].attribute_value_id
    );
    assert_eq!(
        "/resource_value/vpcId",
        subnet1_vpc_subscriptions[0].path.to_string()
    );

    // Verify values flow through subscriptions
    let subnet1_region_value = AttributeValue::get_by_id(ctx, subnet1_region_av_id)
        .await?
        .value(ctx)
        .await?;
    let subnet1_vpc_value = AttributeValue::get_by_id(ctx, subnet1_vpc_av_id)
        .await?
        .value(ctx)
        .await?;

    assert_eq!(Some(serde_json::json!("us-west-2")), subnet1_region_value);
    assert_eq!(Some(serde_json::json!("vpc-12345")), subnet1_vpc_value);

    Ok(())
}

/// Create a "subscription_tracker" component "
async fn create_subscription_tracker_asset(ctx: &DalContext) -> Result<SchemaVariantId> {
    let subscription_tracker = variant::create(
        ctx,
        "subscription_tracker",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Value", kind: "string" },
                        { name: "Sources", kind: "json" },
                    ],
                };
            }
        "#,
    )
    .await?;
    variant::create_management_func(
        ctx,
        "subscription_tracker",
        "save_subscriptions",
        r##"
            async function main({ thisComponent, components }: Input): Promise<Output> {
                let component_sources = Object.fromEntries(Object.values(components).map(
                    (component) => [
                        component.properties.si.name,
                        {
                            attributes: {
                                "/domain/Sources": component.sources,
                            }
                        }
                    ]
                ));
                return {
                    status: "ok",
                    ops: {
                        update: {
                            self: {
                                attributes: {
                                    "/domain/Sources": thisComponent.sources,
                                }
                            },
                            ...component_sources
                        }
                    }
                };
            }
        "##,
    )
    .await?;
    Ok(subscription_tracker)
}
