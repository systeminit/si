use std::collections::HashSet;

use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    SchemaId,
    SchemaVariantId,
    Ulid,
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
            ManagementPrototype,
            ManagementPrototypeError,
            ManagementPrototypeExecution,
        },
    },
};
use dal_test::{
    Report,
    Result,
    SCHEMA_ID_SMALL_EVEN_LEGO,
    expected::{
        ExpectComponent,
        ExpectComponentInputSocket,
        ExpectSchemaVariant,
        ExpectView,
    },
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        change_set,
        component,
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
use si_frontend_types::RawGeometry;
use si_id::ViewId;
use tokio::try_join;
use veritech_client::{
    ManagementFuncStatus,
    ResourceStatus,
};

async fn find_mgmt_prototype(
    ctx: &DalContext,
    component_id: ComponentId,
    prototype_name: &str,
) -> Result<ManagementPrototype> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, schema_variant_id)
        .await?
        .into_iter()
        .find(|proto| proto.name() == prototype_name)
        .expect("could not find prototype");

    Ok(management_prototype)
}

async fn exec_mgmt_func(
    ctx: &DalContext,
    component_id: ComponentId,
    prototype_name: &str,
    view_id: Option<ViewId>,
) -> Result<(ManagementPrototypeExecution, ManagementFuncReturn)> {
    let management_prototype = find_mgmt_prototype(ctx, component_id, prototype_name).await?;

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

async fn exec_mgmt_func_and_operate(
    ctx: &DalContext,
    component_id: ComponentId,
    prototype_name: &str,
    view_id: Option<ViewId>,
) -> Result<()> {
    let (execution_result, result) =
        exec_mgmt_func(ctx, component_id, prototype_name, view_id).await?;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        component_id,
        operations,
        execution_result,
        None,
        ulid::Ulid::new(),
    )
    .await?
    .operate()
    .await?;

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
async fn create_and_connect_to_self_as_children(ctx: &mut DalContext) -> Result<()> {
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

    let new_component_count = 3;
    let string_count = format!("{new_component_count}");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(string_count))).await?;

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Create and Connect to Self as Children",
        None,
    )
    .await?;

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

    let geometry = small_odd_lego
        .geometry(ctx, View::get_id_for_default(ctx).await?)
        .await?;

    assert_eq!(Some(500), geometry.width());
    assert_eq!(Some(500), geometry.height());

    let components = Component::list(ctx).await?;
    assert_eq!(4, components.len());

    let children: HashSet<_> = Component::get_children_for_id(ctx, small_odd_lego.id())
        .await?
        .into_iter()
        .collect();
    assert_eq!(3, children.len());
    let managed: HashSet<_> = small_odd_lego.get_managed(ctx).await?.into_iter().collect();
    assert_eq!(children, managed);

    let small_even_lego_schema_id: SchemaId =
        ulid::Ulid::from_string(SCHEMA_ID_SMALL_EVEN_LEGO)?.into();

    for &child_id in &children {
        let c = Component::get_by_id(ctx, child_id).await?;
        let schema_id = c.schema(ctx).await?.id();
        assert_eq!(small_even_lego_schema_id, schema_id);
    }

    // Ensure parallel edges make it through the rebase
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    let children_base: HashSet<_> = Component::get_children_for_id(ctx, small_odd_lego.id())
        .await?
        .into_iter()
        .collect();
    assert_eq!(3, children_base.len());
    let managed_base: HashSet<_> = small_odd_lego.get_managed(ctx).await?.into_iter().collect();

    assert_eq!(children, children_base);
    assert_eq!(children_base, managed_base);

    Ok(())
}

#[test]
async fn create_and_connect_to_self(ctx: &DalContext) -> Result<()> {
    let mut small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;
    let view_id = View::get_id_for_default(ctx).await?;

    let manager_x: isize = 123;
    let manager_y: isize = 346;

    small_odd_lego
        .set_geometry(
            ctx,
            view_id,
            manager_x,
            manager_y,
            None::<isize>,
            None::<isize>,
        )
        .await?;

    let av_id = Component::attribute_value_for_prop(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await?;

    let new_component_count = 3;
    let string_count = format!("{new_component_count}");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(string_count))).await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Create and Connect to Self", None).await?;

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

    let components = Component::list(ctx).await?;
    assert_eq!(4, components.len());

    let connections = small_odd_lego.incoming_connections(ctx).await?;
    assert_eq!(3, connections.len());
    let small_even_lego_schema_id: SchemaId =
        ulid::Ulid::from_string(SCHEMA_ID_SMALL_EVEN_LEGO)?.into();
    let manager_geometry = small_odd_lego.geometry(ctx, view_id).await?.into_raw();
    for connection in connections {
        let c = Component::get_by_id(ctx, connection.from_component_id).await?;

        let c_geo = c.geometry(ctx, view_id).await?.into_raw();
        assert_eq!(manager_x, manager_geometry.x);
        assert_eq!(manager_y, manager_geometry.y);
        assert_eq!(manager_geometry.x + 10, c_geo.x);
        assert_eq!(manager_geometry.y + 10, c_geo.y);

        let schema_id = c.schema(ctx).await?.id();
        assert_eq!(small_even_lego_schema_id, schema_id);
    }

    Ok(())
}

#[test]
async fn create_and_connect_from_self(ctx: &DalContext) -> Result<()> {
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

    let new_component_count = 3;
    let string_count = format!("{new_component_count}");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(string_count))).await?;

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Create and Connect From Self",
        None,
    )
    .await?;

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

    let components = Component::list(ctx).await?;
    assert_eq!(4, components.len());

    let connections = small_odd_lego.outgoing_connections(ctx).await?;
    assert_eq!(3, connections.len());
    let small_even_lego_schema_id: SchemaId =
        ulid::Ulid::from_string(SCHEMA_ID_SMALL_EVEN_LEGO)?.into();
    for connection in connections {
        let c = Component::get_by_id(ctx, connection.to_component_id).await?;
        let schema_id = c.schema(ctx).await?.id();
        assert_eq!(small_even_lego_schema_id, schema_id);
    }

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
async fn deeply_nested_children(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Deeply Nested Children", None).await?;
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

    let mut component_names = vec![];

    let mut current = small_odd_lego.id();
    loop {
        let children = Component::get_children_for_id(ctx, current).await?;

        if children.is_empty() {
            break;
        }

        let child_id = children[0];
        current = child_id;
        let name = Component::get_by_id(ctx, child_id).await?.name(ctx).await?;

        component_names.push(name);
    }

    assert_eq!(
        vec![
            "clone_0", "clone_1", "clone_2", "clone_3", "clone_4", "clone_5", "clone_6", "clone_7",
            "clone_8", "clone_9",
        ],
        component_names
    );

    Ok(())
}

#[test]
async fn override_values_set_by_sockets(ctx: &DalContext) -> Result<()> {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await?;

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Override Props", None).await?;
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

    let current = small_odd_lego.id();
    let children = Component::get_children_for_id(ctx, current).await?;
    assert_eq!(children.len(), 1);
    let child_id = children[0];
    let component = Component::get_by_id(ctx, child_id).await?;

    let props = component.domain_prop_attribute_value(ctx).await?;
    let view = AttributeValue::view(ctx, props).await?;
    assert!(view.is_some());

    let one_av_id =
        Component::attribute_value_for_prop(ctx, component.id(), &["root", "domain", "one"])
            .await?;

    assert!(!AttributeValue::is_set_by_dependent_function(ctx, one_av_id).await?);

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

struct SmallOddLego {
    component: ExpectComponent,
    arity_one: ExpectComponentInputSocket,
    one: ExpectComponentInputSocket,
}

impl SmallOddLego {
    async fn create(ctx: &mut DalContext) -> Self {
        let component = ExpectComponent::create(ctx, "small odd lego").await;
        Self {
            component,
            arity_one: component.input_socket(ctx, "arity_one").await,
            one: component.input_socket(ctx, "one").await,
        }
    }
    async fn create_and_connect_to_inputs(
        &self,
        ctx: &mut DalContext,
    ) -> Result<ExpectComponentInputSocket> {
        exec_mgmt_func_and_operate(
            ctx,
            self.component.id(),
            "Create and Connect to Inputs",
            None,
        )
        .await?;
        Ok(ExpectComponent::find(ctx, "lego")
            .await
            .input_socket(ctx, "two")
            .await)
    }

    async fn connect_to_inputs(&self, ctx: &mut DalContext) -> Result<()> {
        exec_mgmt_func_and_operate(ctx, self.component.id(), "Connect to Inputs", None).await
    }

    async fn disconnect_from_inputs(&self, ctx: &mut DalContext) -> Result<()> {
        exec_mgmt_func_and_operate(ctx, self.component.id(), "Disconnect from Inputs", None).await
    }

    async fn get_input_values(&self, ctx: &mut DalContext) -> Result<serde_json::Value> {
        exec_mgmt_func_and_operate(ctx, self.component.id(), "Get Input Values", None).await?;
        Ok(self
            .component
            .prop(ctx, ["root", "domain", "test_result"])
            .await
            .view(ctx)
            .await
            .expect("No test_result value"))
    }
}

#[test]
async fn create_connect_input_sockets(ctx: &mut DalContext) -> Result<()> {
    // Manager component
    let manager = SmallOddLego::create(ctx).await;

    // External component to connect to the manager
    let external = ExpectComponent::create_named(ctx, "large even lego", "external").await;
    let one = external.output_socket(ctx, "one").await;
    let three = external.output_socket(ctx, "three").await;
    let five = external.output_socket(ctx, "five").await;

    // Create lego from inputs arity_one=one, two=[three,five]
    one.connect(ctx, manager.arity_one).await;
    three.connect(ctx, manager.one).await;
    five.connect(ctx, manager.one).await;
    let lego = manager.create_and_connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![one, three, five]);

    Ok(())
}

#[test]
async fn update_connect_add_input_sockets(ctx: &mut DalContext) -> Result<()> {
    // Manager component
    let manager = SmallOddLego::create(ctx).await;

    // External component to connect to the manager
    let external = ExpectComponent::create_named(ctx, "large even lego", "external").await;
    let one = external.output_socket(ctx, "one").await;
    let three = external.output_socket(ctx, "three").await;
    let five = external.output_socket(ctx, "five").await;

    // Create empty lego
    let lego = manager.create_and_connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![]);

    // Connect lego to inputs arity_one=one, two=[three,five]
    one.connect(ctx, manager.arity_one).await;
    three.connect(ctx, manager.one).await;
    five.connect(ctx, manager.one).await;
    manager.connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![one, three, five]);

    Ok(())
}

#[test]
async fn update_connect_remove_input_sockets(ctx: &mut DalContext) -> Result<()> {
    // Manager component
    let manager = SmallOddLego::create(ctx).await;

    // External component to connect to the manager
    let external = ExpectComponent::create_named(ctx, "large even lego", "external").await;
    let one = external.output_socket(ctx, "one").await;
    let three = external.output_socket(ctx, "three").await;
    let five = external.output_socket(ctx, "five").await;

    // Create lego from inputs arity_one=one, two=[three,five]
    one.connect(ctx, manager.arity_one).await;
    three.connect(ctx, manager.one).await;
    five.connect(ctx, manager.one).await;
    let lego = manager.create_and_connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![one, three, five]);

    // Disconnect from all inputs
    manager.disconnect_from_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![]);

    Ok(())
}

#[test]
async fn connect_input_sockets_redundant(ctx: &mut DalContext) -> Result<()> {
    // Manager component
    let manager = SmallOddLego::create(ctx).await;

    // External component to connect to the manager
    let external = ExpectComponent::create_named(ctx, "large even lego", "external").await;
    let one = external.output_socket(ctx, "one").await;
    let three = external.output_socket(ctx, "three").await;
    let five = external.output_socket(ctx, "five").await;

    // Create empty lego
    let lego = manager.create_and_connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![]);

    // Connect all inputs to all sockets (including arity_one and one, which means we'll pass
    // the same input twice and it should only connect once)
    one.connect(ctx, manager.arity_one).await;
    one.connect(ctx, manager.one).await;
    three.connect(ctx, manager.one).await;
    five.connect(ctx, manager.one).await;

    // Disconnect from all inputs even though there are no inputs to disconnect from
    manager.disconnect_from_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![]);

    // Connect to all inputs
    manager.connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![one, three, five]);

    // Connect to all inputs again even though we've already connected to them all
    manager.connect_to_inputs(ctx).await?;
    assert_eq!(lego.connections(ctx).await, vec![one, three, five]);

    Ok(())
}

#[test]
async fn get_input_socket_values(ctx: &mut DalContext) -> Result<()> {
    // Manager component
    let manager = SmallOddLego::create(ctx).await;

    // External component to connect to the manager
    let external = ExpectComponent::create_named(ctx, "large even lego", "external").await;
    let one = external.output_socket(ctx, "one").await;
    let three = external.output_socket(ctx, "three").await;
    let five = external.output_socket(ctx, "five").await;
    external
        .prop(ctx, ["root", "domain", "one"])
        .await
        .set(ctx, json!("one"))
        .await;
    external
        .prop(ctx, ["root", "domain", "three"])
        .await
        .set(ctx, "three")
        .await;
    external
        .prop(ctx, ["root", "domain", "five"])
        .await
        .set(ctx, "five")
        .await;

    // Create empty lego
    manager.create_and_connect_to_inputs(ctx).await?;
    change_set::commit(ctx).await?; // wait for dvu
    assert_eq!(manager.get_input_values(ctx).await?, json!({ "one": [] }));

    // Connect all sockets
    one.connect(ctx, manager.arity_one).await;
    three.connect(ctx, manager.one).await;
    five.connect(ctx, manager.one).await;
    manager.connect_to_inputs(ctx).await?;
    change_set::commit(ctx).await?; // wait for dvu
    assert_eq!(
        manager.get_input_values(ctx).await?,
        json!({ "arity_one": "one", "one": ["five", "three"] })
    );

    // Connect one socket redundantly
    one.connect(ctx, manager.one).await;
    manager.connect_to_inputs(ctx).await?;
    change_set::commit(ctx).await?; // wait for dvu
    assert_eq!(
        manager.get_input_values(ctx).await?,
        json!({ "arity_one": "one", "one": ["five", "one", "three"] })
    );

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
async fn incoming_connections_inferred_from_parent(ctx: DalContext) -> Result<()> {
    // Create a manager with inferred connection to parent value
    let mut test = connection_test::setup(ctx).await?;
    let parent = test.create_input("parent", None).await?;
    test.set(parent, "Value", "parent").await;
    let manager = test.create_output("manager", parent).await?;
    test.commit().await?;
    assert_eq!(
        manager.domain(&test.ctx).await,
        json!({ "Value": "parent" })
    );

    // Call management function to create component with inferred parent connections.
    let component = test.create_output_and_copy_connection(manager).await?;
    test.commit().await?;

    // Check that value propagated from parent to connection
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "parent" })
    );

    // Check that the connection is real: delete manager and update parent value, and see if it
    // propagates.
    manager.component(&test.ctx).await.delete(&test.ctx).await?;
    test.set(parent, "Value", "new_parent").await;
    test.commit().await?;
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "new_parent" })
    );

    Ok(())
}

#[test]
async fn incoming_connections_inferred_multiple_ancestors(ctx: DalContext) -> Result<()> {
    // Create a manager with inferred connection to parent value
    let mut test = connection_test::setup(ctx).await?;
    let parent = test.create_input("parent", None).await?;
    let parent2 = test.create_input2("parent2", parent).await?;
    test.set(parent, "Value", "parent").await;
    test.set(parent2, "Value2", "parent2").await;
    let manager = test.create_output("manager", parent2).await?;
    test.commit().await?;
    assert_eq!(
        manager.domain(&test.ctx).await,
        json!({ "Value": "parent", "Value2": "parent2" })
    );

    // Call management function to create component with inferred parent connections.
    let component = test.create_output_and_copy_connection(manager).await?;
    test.commit().await?;
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "parent", "Value2": "parent2" })
    );

    // Check that the connection is real: delete manager and update parent value, and see if it
    // propagates.
    manager.component(&test.ctx).await.delete(&test.ctx).await?;
    test.set(parent, "Value", "new_parent").await;
    test.set(parent2, "Value2", "new_parent2").await;
    test.commit().await?;
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "new_parent", "Value2": "new_parent2" })
    );

    Ok(())
}

#[test]
async fn incoming_connections_none(ctx: DalContext) -> Result<()> {
    // Create a manager with inferred connection
    let mut test = connection_test::setup(ctx).await?;
    let manager = test.create_output("manager", None).await?;
    test.commit().await?;
    assert_eq!(manager.domain(&test.ctx).await, json!({}));

    // Call management function to create component without incomingConnections.
    let component = test.create_output_and_copy_connection(manager).await?;
    test.commit().await?;
    assert_eq!(component.domain(&test.ctx).await, json!({}));

    Ok(())
}

#[test]
async fn component_incoming_connections(ctx: DalContext) -> Result<()> {
    // Create a manager with inferred connection to parent value
    let mut test = connection_test::setup(ctx).await?;
    let input = test.create_input("input", None).await?;
    test.set(input, "Value", "input").await;
    let component = test.create_output("component", None).await?;
    input.connect(&test.ctx, "Value", component, "Value").await;
    let manager = test.create_output("manager", None).await?;
    Component::manage_component(&test.ctx, manager.id(), component.id()).await?;
    test.commit().await?;
    // Check that value propagated from input to component
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "input" })
    );

    // Call management function to count and delete component incoming connections.
    test.remove_all_connections(manager).await;
    test.commit().await?;

    // Check that the connection was passed and removed
    test.set(input, "Value", "new_input").await;
    test.commit().await?;
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "IncomingConnectionsCount": 1 })
    );

    Ok(())
}

#[test]
async fn component_incoming_connections_inferred_from_parent(ctx: DalContext) -> Result<()> {
    // Create a manager with inferred connection to parent value
    let mut test = connection_test::setup(ctx).await?;
    let parent = test.create_input("parent", None).await?;
    test.set(parent, "Value", "parent").await;
    let component = test.create_output("component", parent).await?;
    let manager = test.create_output("manager", None).await?;
    Component::manage_component(&test.ctx, manager.id(), component.id()).await?;
    test.commit().await?;
    // Check that value propagated from parent to component
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "parent" })
    );

    // Call management function to count and delete component incoming connections.
    test.remove_all_connections(manager).await;
    test.commit().await?;

    // Check that the connection was passed but the connection was not removed
    test.set(parent, "Value", "new_parent").await;
    test.commit().await?;
    assert_eq!(
        component.domain(&test.ctx).await,
        json!({ "Value": "new_parent", "IncomingConnectionsCount": 1 })
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
            "Sources": {
                "/domain/Value": { "component": component::id(ctx, "NewComponent").await?, "path": "/domain/Value" }
            }
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from source",
            "Sources": {
                "/domain/Value": { "component": component::id(ctx, "source").await?, "path": "/domain/Value" }
            }
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
            "Sources": {}
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": {
                "/domain/Value": { "component": component::id(ctx, "Bar").await?, "path": "/domain/Value" }
            }
        }),
        component::domain(ctx, "Foo").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": {}
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
            "Sources": {}
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Foo",
            "Sources": {}
        }),
        component::domain(ctx, "Foo").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Foo",
            "Sources": {
                "/domain/Value": { "component": component::id(ctx, "Foo").await?, "path": "/domain/Value" }
            }
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
            "Sources": {}
        }),
        component::domain(ctx, "manager").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": {
                "/domain/Value": { "component": component::id(ctx, "Bar").await?, "path": "/domain/Value" }
            }
        }),
        component::domain(ctx, "Foo").await?,
    );
    assert_eq!(
        json!({
            "Value": "value from Bar",
            "Sources": {}
        }),
        component::domain(ctx, "Bar").await?,
    );

    Ok(())
}

pub mod connection_test {
    use dal::{
        ComponentType,
        DalContext,
    };
    use dal_test::{
        Result,
        expected::{
            ExpectComponent,
            ExpectFunc,
            ExpectSchemaVariant,
        },
        helpers::{
            change_set,
            component,
            schema::variant,
        },
    };
    use serde_json::Value;

    pub async fn setup(ctx: DalContext) -> Result<ConnectionTest> {
        // "input" with Value output socket.
        let input = ExpectSchemaVariant::create_named(
            &ctx,
            "input",
            r#"
                function main() {
                    return {
                        props: [
                            { name: "Value", kind: "string" },
                        ],
                        outputSockets: [
                            { name: "Value", arity: "many", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Value" ] }, connectionAnnotations: "[\"Value\"]" },
                        ],
                    };
                }
            "#,
        ).await;
        input
            .set_type(&ctx, ComponentType::ConfigurationFrameDown)
            .await;

        // "input2" with Value output socket.
        let input2 = ExpectSchemaVariant::create_named(
            &ctx,
            "input2",
            r#"
                function main() {
                    return {
                        props: [
                            { name: "Value2", kind: "string" },
                        ],
                        outputSockets: [
                            { name: "Value2", arity: "many", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Value2" ] }, connectionAnnotations: "[\"Value2\"]" },
                        ],
                    };
                }
            "#,
        ).await;
        input2
            .set_type(&ctx, ComponentType::ConfigurationFrameDown)
            .await;

        // "output" with Value and Value2 input sockets.
        let output = ExpectSchemaVariant::create_named(&ctx, "output", r#"
            function main() {
                return {
                    inputSockets: [
                        { name: "Value", arity: "one", connectionAnnotations: "[\"Value\"]" },
                        { name: "Value2", arity: "one", connectionAnnotations: "[\"Value2\"]" },
                    ],
                    props: [
                        { name: "Value", kind: "string", valueFrom: { kind: "inputSocket", socket_name: "Value" } },
                        { name: "Value2", kind: "string", valueFrom: { kind: "inputSocket", socket_name: "Value2" } },
                        { name: "IncomingConnectionsCount", kind: "integer" },
                    ],
                };
            }
        "#).await;

        // Management func that creates a new component connected to our input
        let create_output_and_copy_connection = variant::create_management_func(
            &ctx,
            "output",
            "create_output_and_copy_connection",
            r#"
                    async function main({ thisComponent }: Input): Promise<Output> {
                        let connect = [];
                        if (thisComponent.incomingConnections.Value) {
                            connect.push({
                                from: thisComponent.incomingConnections.Value,
                                to: "Value"
                            });
                        }
                        if (thisComponent.incomingConnections.Value2) {
                            connect.push({
                                from: thisComponent.incomingConnections.Value2,
                                to: "Value2"
                            });
                        }
                        return {
                            status: "ok",
                            ops: {
                                create: {
                                    output: {
                                        kind: "output",
                                        connect
                                    }
                                }
                            }
                        }
                    }
                "#,
        )
        .await?
        .into();

        // Management func that creates a new component connected to our input
        let remove_all_connections = variant::create_management_func(&ctx, "output",
        "remove_all_connections",
                r#"
                    async function main({ components }: Input): Promise<Output> {
                        function updateComponent(component: Input["components"][string]) {
                            let connections = Object.entries(component.incomingConnections).flatMap(([to, from]) => {
                                if (Array.isArray(from)) {
                                    return from.map((from) => ({ to, from }));
                                } else if (from) {
                                    return [{ to, from }];
                                } else {
                                    return [];
                                }
                            });
                            return {
                                properties: {
                                    domain: {
                                        IncomingConnectionsCount: connections.length,
                                    },
                                },
                                connect: {
                                    remove: connections,
                                },
                            };
                        }

                        return {
                            status: "ok",
                            ops: {
                                update: Object.fromEntries(
                                    Object.entries(components).map(
                                        ([name, component]) => [name, updateComponent(component)]
                                    )
                                ),
                            }
                        }
                    }
                "#,
            )
            .await?
            .into();

        Ok(ConnectionTest {
            ctx,
            input,
            input2,
            output,
            create_output_and_copy_connection,
            remove_all_connections,
        })
    }

    pub struct ConnectionTest {
        pub ctx: DalContext,
        pub input: ExpectSchemaVariant,
        pub input2: ExpectSchemaVariant,
        pub output: ExpectSchemaVariant,
        pub create_output_and_copy_connection: ExpectFunc,
        pub remove_all_connections: ExpectFunc,
    }

    impl ConnectionTest {
        /// Create an input2 component with given (optional) parent
        pub async fn create_input(
            &self,
            name: &str,
            parent: impl Into<Option<ExpectComponent>>,
        ) -> Result<ExpectComponent> {
            let component =
                ExpectComponent(component::create(&self.ctx, self.input.id(), name).await?);
            if let Some(parent) = parent.into() {
                component.upsert_parent(&self.ctx, parent).await;
            }
            Ok(component)
        }

        /// Create an input2 component with given (optional) parent
        pub async fn create_input2(
            &self,
            name: &str,
            parent: impl Into<Option<ExpectComponent>>,
        ) -> Result<ExpectComponent> {
            let component =
                ExpectComponent(component::create(&self.ctx, self.input2.id(), name).await?);
            if let Some(parent) = parent.into() {
                component.upsert_parent(&self.ctx, parent).await;
            }
            Ok(component)
        }

        /// Create an output component with given (optional) parent
        pub async fn create_output(
            &self,
            name: &str,
            parent: impl Into<Option<ExpectComponent>>,
        ) -> Result<ExpectComponent> {
            let component =
                ExpectComponent(component::create(&self.ctx, self.output.id(), name).await?);
            if let Some(parent) = parent.into() {
                component.upsert_parent(&self.ctx, parent).await;
            }
            Ok(component)
        }

        pub async fn set(&self, component: ExpectComponent, prop: &str, value: &str) -> Value {
            component
                .prop(&self.ctx, ["root", "domain", prop])
                .await
                .set(&self.ctx, value)
                .await;
            serde_json::json!(value)
        }

        pub async fn create_output_and_copy_connection(
            &self,
            manager: ExpectComponent,
        ) -> Result<ExpectComponent> {
            manager
                .execute_management_func(&self.ctx, self.create_output_and_copy_connection)
                .await;
            let mut managed = manager
                .component(&self.ctx)
                .await
                .get_managed(&self.ctx)
                .await?;
            assert_eq!(managed.len(), 1);
            Ok(managed
                .pop()
                .expect("should have a managed component")
                .into())
        }

        pub async fn remove_all_connections(&self, manager: ExpectComponent) {
            manager
                .execute_management_func(&self.ctx, self.remove_all_connections)
                .await;
        }

        pub async fn commit(&mut self) -> Result<()> {
            change_set::commit(&mut self.ctx).await?;
            Ok(())
        }
    }
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

    let prototype_id = find_mgmt_prototype(ctx, small_odd_lego.id(), "Clone")
        .await?
        .id();
    let prototype_2_id = find_mgmt_prototype(ctx, small_odd_lego.id(), "Update")
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
    )
    .await
    .expect_err("should not allow you to transition state");

    let new_state = ManagementFuncJobState::transition_state(
        &ctx_clone,
        latest_execution.id(),
        ManagementState::Executing,
        Some(Ulid::new().into()),
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
