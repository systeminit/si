use std::collections::HashSet;

use dal::{
    component::resource::ResourceData,
    diagram::{geometry::Geometry, view::View},
    management::{
        prototype::{ManagementPrototype, ManagementPrototypeExecution},
        ManagementFuncReturn, ManagementGeometry, ManagementOperator,
    },
    AttributeValue, Component, ComponentId, DalContext, SchemaId,
};
use dal_test::{
    expected::{apply_change_set_to_base, ExpectView},
    helpers::ChangeSetTestHelpers,
};
use dal_test::{
    helpers::create_component_for_default_schema_name_in_default_view, test,
    SCHEMA_ID_SMALL_EVEN_LEGO,
};
use si_frontend_types::RawGeometry;
use si_id::ViewId;
use veritech_client::{ManagementFuncStatus, ResourceStatus};

pub mod generator;

async fn exec_mgmt_func(
    ctx: &DalContext,
    component_id: ComponentId,
    prototype_name: &str,
    view_id: Option<ViewId>,
) -> (ManagementPrototypeExecution, ManagementFuncReturn) {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id)
        .await
        .expect("get schema variant id");

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, schema_variant_id)
        .await
        .expect("get prototypes")
        .into_iter()
        .find(|proto| proto.name() == prototype_name)
        .expect("could not find prototype");

    let mut execution_result = management_prototype
        .execute(ctx, component_id, view_id)
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = execution_result
        .result
        .take()
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    (execution_result, result)
}

#[test]
async fn update_managed_components_in_view(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await
    .expect("could not create component");

    let view_name = "a view askew";
    let new_view_id = ExpectView::create_with_name(ctx, view_name).await.id();
    Geometry::new_for_component(ctx, small_odd_lego.id(), new_view_id)
        .await
        .expect("create geometry in view");
    Geometry::new_for_component(ctx, small_even_lego.id(), new_view_id)
        .await
        .expect("create geometry in view");

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id())
        .await
        .expect("add manages edge");

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Update in View",
        Some(new_view_id),
    )
    .await;

    assert_eq!(ManagementFuncStatus::Ok, result.status);
    assert_eq!(Some(view_name), result.message.as_deref());

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        operations,
        execution_result,
        Some(new_view_id),
    )
    .await
    .expect("should create operator")
    .operate()
    .await
    .expect("should operate");

    let mut new_component = None;
    let components = Component::list(ctx).await.expect("list components");
    assert_eq!(2, components.len());
    for c in components {
        if c.name(ctx).await.expect("get name") == "small even lego managed by small odd lego" {
            new_component = Some(c);
            break;
        }
    }

    let new_component = new_component.expect("should have found the cloned component");
    let default_view_id = ExpectView::get_id_for_default(ctx).await;
    let default_view_geometry = new_component
        .geometry(ctx, default_view_id)
        .await
        .expect("get geometry for default view");

    assert_eq!(0, default_view_geometry.x());
    assert_eq!(0, default_view_geometry.y());

    let new_view_geometry = new_component
        .geometry(ctx, new_view_id)
        .await
        .expect("get geo for view askew");

    assert_eq!(1000, new_view_geometry.x());
    assert_eq!(750, new_view_geometry.y());
}

#[test]
async fn update_managed_components(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await
    .expect("could not create component");

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id())
        .await
        .expect("add manages edge");

    let (execution_result, result) = exec_mgmt_func(ctx, small_odd_lego.id(), "Update", None).await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let mut new_component = None;
    let components = Component::list(ctx).await.expect("list components");
    assert_eq!(2, components.len());
    for c in components {
        if c.name(ctx).await.expect("get name") == "small even lego managed by small odd lego" {
            new_component = Some(c);
            break;
        }
    }

    let _new_component = new_component.expect("should have found the cloned component");
}

#[test]
async fn create_component_of_other_schema(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");
    let small_even_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small even lego",
        "small even lego",
    )
    .await
    .expect("could not create component");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_even_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    AttributeValue::update(
        ctx,
        av_id,
        Some(serde_json::json!("small even lego resource id")),
    )
    .await
    .expect("able to update value");

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id())
        .await
        .expect("add manages edge");

    let (execution_result, result) = exec_mgmt_func(ctx, small_odd_lego.id(), "Clone", None).await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let mut new_component = None;
    let components = Component::list(ctx).await.expect("list components");
    assert_eq!(4, components.len());
    for c in components {
        if c.name(ctx).await.expect("get name") == "small even lego_clone" {
            new_component = Some(c);
            break;
        }
    }

    let new_component = new_component.expect("should have found the cloned component");
    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        new_component.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    let av = AttributeValue::get_by_id(ctx, av_id)
        .await
        .expect("get value");

    assert_eq!(
        Some(serde_json::json!("small even lego resource id")),
        av.value(ctx).await.expect("get value")
    );
}

#[test]
async fn create_and_connect_to_self_as_children(ctx: &mut DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    let new_component_count = 3;
    let string_count = format!("{new_component_count}");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(string_count)))
        .await
        .expect("able to update value");

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Create and Connect to Self as Children",
        None,
    )
    .await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let geometry = small_odd_lego
        .geometry(ctx, ExpectView::get_id_for_default(ctx).await)
        .await
        .expect("get geometry");

    assert_eq!(Some(500), geometry.width());
    assert_eq!(Some(500), geometry.height());

    let components = Component::list(ctx).await.expect("get components");
    assert_eq!(4, components.len());

    let children: HashSet<_> = Component::get_children_for_id(ctx, small_odd_lego.id())
        .await
        .expect("get frame children")
        .into_iter()
        .collect();
    assert_eq!(3, children.len());
    let managed: HashSet<_> = small_odd_lego
        .get_managed(ctx)
        .await
        .expect("get managed")
        .into_iter()
        .collect();
    assert_eq!(children, managed);

    let small_even_lego_schema_id: SchemaId = ulid::Ulid::from_string(SCHEMA_ID_SMALL_EVEN_LEGO)
        .expect("make ulid")
        .into();

    for &child_id in &children {
        let c = Component::get_by_id(ctx, child_id)
            .await
            .expect("get component");
        let schema_id = c.schema(ctx).await.expect("get schema").id();
        assert_eq!(small_even_lego_schema_id, schema_id);
    }

    // Ensure parallel edges make it through the rebase
    apply_change_set_to_base(ctx).await;

    let children_base: HashSet<_> = Component::get_children_for_id(ctx, small_odd_lego.id())
        .await
        .expect("get frame children")
        .into_iter()
        .collect();
    assert_eq!(3, children_base.len());
    let managed_base: HashSet<_> = small_odd_lego
        .get_managed(ctx)
        .await
        .expect("get managed")
        .into_iter()
        .collect();

    assert_eq!(children, children_base);
    assert_eq!(children_base, managed_base);
}

#[test]
async fn create_and_connect_to_self(ctx: &DalContext) {
    let mut small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");
    let view_id = View::get_id_for_default(ctx).await.expect("get view id");

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
        .await
        .expect("set geometry");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    let new_component_count = 3;
    let string_count = format!("{new_component_count}");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(string_count)))
        .await
        .expect("able to update value");

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Create and Connect to Self", None).await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let components = Component::list(ctx).await.expect("get components");
    assert_eq!(4, components.len());

    let connections = small_odd_lego
        .incoming_connections(ctx)
        .await
        .expect("get incoming conns");
    assert_eq!(3, connections.len());
    let small_even_lego_schema_id: SchemaId = ulid::Ulid::from_string(SCHEMA_ID_SMALL_EVEN_LEGO)
        .expect("make ulid")
        .into();
    let manager_geometry = small_odd_lego
        .geometry(ctx, view_id)
        .await
        .expect("get geometry")
        .into_raw();
    for connection in connections {
        let c = Component::get_by_id(ctx, connection.from_component_id)
            .await
            .expect("get component");

        let c_geo = c.geometry(ctx, view_id).await.expect("get geo").into_raw();
        assert_eq!(manager_x, manager_geometry.x);
        assert_eq!(manager_y, manager_geometry.y);
        assert_eq!(manager_geometry.x + 10, c_geo.x);
        assert_eq!(manager_geometry.y + 10, c_geo.y);

        let schema_id = c.schema(ctx).await.expect("get schema").id();
        assert_eq!(small_even_lego_schema_id, schema_id);
    }
}

#[test]
async fn create_and_connect_from_self(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    let new_component_count = 3;
    let string_count = format!("{new_component_count}");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(string_count)))
        .await
        .expect("able to update value");

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Create and Connect From Self",
        None,
    )
    .await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let components = Component::list(ctx).await.expect("get components");
    assert_eq!(4, components.len());

    let connections = small_odd_lego
        .outgoing_connections(ctx)
        .await
        .expect("get outgoing conns");
    assert_eq!(3, connections.len());
    let small_even_lego_schema_id: SchemaId = ulid::Ulid::from_string(SCHEMA_ID_SMALL_EVEN_LEGO)
        .expect("make ulid")
        .into();
    for connection in connections {
        let c = Component::get_by_id(ctx, connection.to_component_id)
            .await
            .expect("get component");
        let schema_id = c.schema(ctx).await.expect("get schema").id();
        assert_eq!(small_even_lego_schema_id, schema_id);
    }
}

#[test]
async fn create_component_of_same_schema(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let (execution_result, result) = exec_mgmt_func(ctx, small_odd_lego.id(), "Clone", None).await;
    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let mut new_component = None;
    let components = Component::list(ctx).await.expect("list components");
    assert_eq!(2, components.len());
    for c in components {
        if c.name(ctx).await.expect("get name") == "small odd lego_clone" {
            new_component = Some(c);
            break;
        }
    }

    let default_view_id = ExpectView::get_id_for_default(ctx).await;

    let new_component = new_component.expect("should have found the cloned component");
    let new_geometry: ManagementGeometry = new_component
        .geometry(ctx, default_view_id)
        .await
        .expect("get geometry")
        .into_raw()
        .into();

    assert_eq!(Some(10.0), new_geometry.x);
    assert_eq!(Some(20.0), new_geometry.y);

    let managers = new_component.managers(ctx).await.expect("get managers");

    assert_eq!(1, managers.len());
    assert_eq!(
        small_odd_lego.id(),
        managers[0],
        "should have the same manager"
    );

    let (execution_result, result) = exec_mgmt_func(ctx, small_odd_lego.id(), "Clone", None).await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let mut new_component_2 = None;

    let components = Component::list(ctx).await.expect("list components");
    assert_eq!(4, components.len());
    for c in components {
        let name = c.name(ctx).await.expect("get name");
        if name == "small odd lego_clone_clone" {
            new_component_2 = Some(c);
            //break;
        }
    }
    let _new_component_2 = new_component_2.expect("should have found the cloned component again");
}

#[test]
async fn execute_management_func(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("import id")))
        .await
        .expect("able to update value");

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Import small odd lego", None).await;

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "domain", "two"],
    )
    .await
    .expect("get four");

    let two_av = AttributeValue::get_by_id(ctx, av_id)
        .await
        .expect("a fleetwood to my mac");

    let two_value = two_av.value(ctx).await.expect("get value");

    assert_eq!(Some(serde_json::json!("step")), two_value);
}

#[test]
async fn deeply_nested_children(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Deeply Nested Children", None).await;
    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let mut component_names = vec![];

    let mut current = small_odd_lego.id();
    loop {
        let children = Component::get_children_for_id(ctx, current)
            .await
            .expect("get children");

        if children.is_empty() {
            break;
        }

        let child_id = children[0];
        current = child_id;
        let name = Component::get_by_id(ctx, child_id)
            .await
            .expect("get comp")
            .name(ctx)
            .await
            .expect("get name");

        component_names.push(name);
    }

    assert_eq!(
        vec![
            "clone_0", "clone_1", "clone_2", "clone_3", "clone_4", "clone_5", "clone_6", "clone_7",
            "clone_8", "clone_9",
        ],
        component_names
    )
}

#[test]
async fn override_values_set_by_sockets(ctx: &DalContext) {
    let small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Override Props", None).await;
    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    let current = small_odd_lego.id();
    let children = Component::get_children_for_id(ctx, current)
        .await
        .expect("get children");
    assert_eq!(children.len(), 1);
    let child_id = children[0];
    let component = Component::get_by_id(ctx, child_id).await.expect("get comp");

    let props = component
        .domain_prop_attribute_value(ctx)
        .await
        .expect("could not get domain");
    let domain = AttributeValue::get_by_id(ctx, props)
        .await
        .expect("could not get attribute value");
    let view = domain.view(ctx).await.expect("could not get view");
    assert!(view.is_some());
}

#[test]
async fn create_in_views(ctx: &DalContext) {
    let mut small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let default_view_id = ExpectView::get_id_for_default(ctx).await;

    let manager_x = 50;
    let manager_y = 75;

    small_odd_lego
        .set_geometry(ctx, default_view_id, manager_x, manager_y, None, None)
        .await
        .expect("set manager component geometry");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    let view_name = "the black lodge";
    let view = ExpectView::create_with_name(ctx, view_name).await;

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(view_name)))
        .await
        .expect("able to update value");

    let (execution_result, result) =
        exec_mgmt_func(ctx, small_odd_lego.id(), "Create in Other Views", None).await;

    let operations = result.operations.expect("should have operations");

    let component_id =
        ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
            .await
            .expect("should create operator")
            .operate()
            .await
            .expect("should operate")
            .expect("should return component ids")
            .pop()
            .expect("should have a component id");

    let geometries = Geometry::by_view_for_component_id(ctx, component_id)
        .await
        .expect("get geometries");

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
}

#[test]
async fn create_view_and_in_view(ctx: &DalContext) {
    let mut small_odd_lego = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd lego",
    )
    .await
    .expect("could not create component");

    let default_view_id = ExpectView::get_id_for_default(ctx).await;

    let manager_x = 50;
    let manager_y = 75;

    small_odd_lego
        .set_geometry(ctx, default_view_id, manager_x, manager_y, None, None)
        .await
        .expect("set manager component geometry");

    let av_id = Component::attribute_value_for_prop_by_id(
        ctx,
        small_odd_lego.id(),
        &["root", "si", "resourceId"],
    )
    .await
    .expect("av should exist");

    let view_name = "the red room";
    AttributeValue::update(ctx, av_id, Some(serde_json::json!(view_name)))
        .await
        .expect("able to update value");

    let (execution_result, result) = exec_mgmt_func(
        ctx,
        small_odd_lego.id(),
        "Create View and Component in View",
        None,
    )
    .await;

    let operations = result.operations.expect("should have operations");

    let component_id =
        ManagementOperator::new(ctx, small_odd_lego.id(), operations, execution_result, None)
            .await
            .expect("should create operator")
            .operate()
            .await
            .expect("should operate")
            .expect("should return component ids")
            .pop()
            .expect("should have a component id");

    let red_room = View::find_by_name(ctx, view_name)
        .await
        .expect("find view")
        .expect("view exists");

    let geometries = Geometry::by_view_for_component_id(ctx, component_id)
        .await
        .expect("get geometries");

    assert_eq!(1, geometries.len());

    let red_room_geo = geometries
        .get(&red_room.id())
        .cloned()
        .expect("has a geometry in the red room");

    assert_eq!(315, red_room_geo.x() - manager_x);
    assert_eq!(315, red_room_geo.y() - manager_y);
}

#[test]
async fn delete_and_erase_components(ctx: &mut DalContext) {
    let manager = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "middle manager",
    )
    .await
    .expect("could not create component");

    let component_with_resource_to_delete =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to delete",
        )
        .await
        .expect("could not create component");

    let component_still_on_head = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component still on head",
    )
    .await
    .expect("could not create component");

    let component_with_resource_to_erase =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to erase",
        )
        .await
        .expect("could not create component");

    for component_id in [
        component_with_resource_to_delete.id(),
        component_still_on_head.id(),
        component_with_resource_to_erase.id(),
    ] {
        Component::manage_component(ctx, manager.id(), component_id)
            .await
            .expect("failed to create management edge");
    }

    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");

    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );

    component_with_resource_to_delete
        .set_resource(ctx, resource_data.clone())
        .await
        .expect("failed to set resource");
    component_with_resource_to_erase
        .set_resource(ctx, resource_data.clone())
        .await
        .expect("failed to set resource");

    let component_to_delete = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component to delete",
    )
    .await
    .expect("could not create component");

    Component::manage_component(ctx, manager.id(), component_to_delete.id())
        .await
        .expect("failed to create management edge");

    let av_id =
        Component::attribute_value_for_prop_by_id(ctx, manager.id(), &["root", "si", "resourceId"])
            .await
            .expect("av should exist");

    let resource_id = format!(
        "{},{},{},{}",
        component_to_delete.name(ctx).await.expect("get name"),
        component_with_resource_to_delete
            .name(ctx)
            .await
            .expect("get name"),
        component_still_on_head.name(ctx).await.expect("get name"),
        component_with_resource_to_erase
            .name(ctx)
            .await
            .expect("get name")
    );

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(resource_id)))
        .await
        .expect("able to update value");

    let (execution_result, result) =
        exec_mgmt_func(ctx, manager.id(), "Delete and Erase", None).await;
    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, manager.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    assert!(
        Component::try_get_by_id(ctx, component_to_delete.id())
            .await
            .expect("should succeed")
            .is_none(),
        "deleted component should be gone"
    );

    assert!(
        Component::try_get_by_id(ctx, component_still_on_head.id())
            .await
            .expect("should succeed")
            .is_none(),
        "deleted component that is still on head should be gone in this change set"
    );

    assert!(
        Component::exists_on_head(ctx, &[component_still_on_head.id()])
            .await
            .expect("should be able to check for components on head")
            .contains(&component_still_on_head.id()),
        "component should still exist on head"
    );

    assert!(
        Component::try_get_by_id(ctx, component_with_resource_to_erase.id())
            .await
            .expect("should be able to look for component")
            .is_none(),
        "erased component should be gone"
    );

    let component_with_resource_to_delete =
        Component::get_by_id(ctx, component_with_resource_to_delete.id())
            .await
            .expect("component with resource should still exist");
    assert!(
        component_with_resource_to_delete.to_delete(),
        "component with resource should be marked as to delete"
    );
}

#[test]
async fn remove_view_and_component_from_view(ctx: &DalContext) {
    let manager =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "c-suite")
            .await
            .expect("could not create component");

    let new_view_name = "vista del mar";
    let new_view = ExpectView::create_with_name(ctx, new_view_name).await;

    let component_in_both_views_1 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "both views 1",
    )
    .await
    .expect("could not create component");

    let component_in_both_views_2 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "both views 2",
    )
    .await
    .expect("could not create component");

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

        Component::manage_component(ctx, manager.id(), component_id)
            .await
            .expect("could not manage component");

        Component::add_to_view(ctx, component_id, new_view.id(), raw_geometry)
            .await
            .expect("could not add to view");
    }

    let av_id =
        Component::attribute_value_for_prop_by_id(ctx, manager.id(), &["root", "si", "resourceId"])
            .await
            .expect("av should exist");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!(new_view_name)))
        .await
        .expect("able to update value");

    let (execution_result, result) =
        exec_mgmt_func(ctx, manager.id(), "Remove View and Components", None).await;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, manager.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("should operate");

    assert!(View::find_by_name(ctx, new_view_name)
        .await
        .expect("could not search views by name")
        .is_none());

    for component_id in [
        component_in_both_views_1.id(),
        component_in_both_views_2.id(),
    ] {
        let geometries = Geometry::by_view_for_component_id(ctx, component_id)
            .await
            .expect("get geometries by view for component");
        assert!(!geometries.is_empty());
        assert!(!geometries.contains_key(&new_view.id()));
    }

    // try to delete the default view
    let av_id =
        Component::attribute_value_for_prop_by_id(ctx, manager.id(), &["root", "si", "resourceId"])
            .await
            .expect("av should exist");

    AttributeValue::update(ctx, av_id, Some(serde_json::json!("DEFAULT")))
        .await
        .expect("able to update value");

    let default_view_id = ExpectView::get_id_for_default(ctx).await;

    let (execution_result, result) =
        exec_mgmt_func(ctx, manager.id(), "Remove View and Components", None).await;

    assert_eq!(ManagementFuncStatus::Ok, result.status);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(ctx, manager.id(), operations, execution_result, None)
        .await
        .expect("should create operator")
        .operate()
        .await
        .expect("Should succeed, even though the view will not be removed since the removal would orphan the manager");

    assert_eq!(default_view_id, ExpectView::get_id_for_default(ctx).await);

    for component_id in [
        component_in_both_views_1.id(),
        component_in_both_views_2.id(),
    ] {
        let geometries = Geometry::by_view_for_component_id(ctx, component_id)
            .await
            .expect("get geometries by view for component");
        assert!(!geometries.is_empty());
    }
}
