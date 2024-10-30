use std::collections::HashMap;

use dal::{
    management::{
        prototype::ManagementPrototype, ManagementFuncReturn, ManagementOperator, NumericGeometry,
    },
    AttributeValue, Component, DalContext,
};
use dal_test::{helpers::create_component_for_default_schema_name, test};
use veritech_client::ManagementFuncStatus;

#[test]
async fn update_managed_components(ctx: &DalContext) {
    let small_odd_lego =
        create_component_for_default_schema_name(ctx, "small odd lego", "small odd lego")
            .await
            .expect("could not create component");
    let small_even_lego =
        create_component_for_default_schema_name(ctx, "small even lego", "small even lego")
            .await
            .expect("could not create component");

    Component::manage_component(ctx, small_odd_lego.id(), small_even_lego.id())
        .await
        .expect("add manages edge");

    let manager_variant = small_odd_lego
        .schema_variant(ctx)
        .await
        .expect("get variant");

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, manager_variant.id())
        .await
        .expect("get prototypes")
        .into_iter()
        .find(|proto| proto.name() == "Update")
        .expect("could not find prototype");

    let execution_result = management_prototype
        .execute(ctx, small_odd_lego.id())
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = execution_result
        .result
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        execution_result.manager_component_geometry,
        operations,
        execution_result.managed_schema_map,
        execution_result.placeholders,
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

    let _new_component = new_component.expect("should have found the cloned component");
}

#[test]
async fn create_component_of_other_schema(ctx: &DalContext) {
    let small_odd_lego =
        create_component_for_default_schema_name(ctx, "small odd lego", "small odd lego")
            .await
            .expect("could not create component");
    let small_even_lego =
        create_component_for_default_schema_name(ctx, "small even lego", "small even lego")
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

    let manager_variant = small_odd_lego
        .schema_variant(ctx)
        .await
        .expect("get variant");

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, manager_variant.id())
        .await
        .expect("get prototypes")
        .into_iter()
        .find(|proto| proto.name() == "Clone")
        .expect("could not find prototype");

    let execution_result = management_prototype
        .execute(ctx, small_odd_lego.id())
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = execution_result
        .result
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        execution_result.manager_component_geometry,
        operations,
        execution_result.managed_schema_map,
        execution_result.placeholders,
    )
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
async fn create_component_of_same_schema(ctx: &DalContext) {
    let small_odd_lego =
        create_component_for_default_schema_name(ctx, "small odd lego", "small odd lego")
            .await
            .expect("could not create component");
    let variant = small_odd_lego
        .schema_variant(ctx)
        .await
        .expect("get variant");

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, variant.id())
        .await
        .expect("get prototypes")
        .into_iter()
        .find(|proto| proto.name() == "Clone")
        .expect("could not find prototype");

    let execution_result = management_prototype
        .execute(ctx, small_odd_lego.id())
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = execution_result
        .result
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        execution_result.manager_component_geometry,
        operations,
        execution_result.managed_schema_map,
        execution_result.placeholders,
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
        if c.name(ctx).await.expect("get name") == "small odd lego_clone" {
            new_component = Some(c);
            break;
        }
    }

    let new_component = new_component.expect("should have found the cloned component");
    let new_geometry: NumericGeometry = new_component
        .geometry(ctx)
        .await
        .expect("get geometry")
        .into_raw()
        .into();

    assert_eq!(10.0, new_geometry.x);
    assert_eq!(20.0, new_geometry.y);

    let managers = new_component.get_managers(ctx).await.expect("get managers");

    assert_eq!(1, managers.len());
    assert_eq!(
        small_odd_lego.id(),
        managers[0],
        "should have the same manager"
    );

    let execution_result = management_prototype
        .execute(ctx, small_odd_lego.id())
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = execution_result
        .result
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        execution_result.manager_component_geometry,
        operations,
        execution_result.managed_schema_map,
        HashMap::new(),
    )
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
    let small_odd_lego =
        create_component_for_default_schema_name(ctx, "small odd lego", "small odd lego")
            .await
            .expect("could not create component");
    let variant = small_odd_lego
        .schema_variant(ctx)
        .await
        .expect("get variant");

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

    let management_prototype = ManagementPrototype::list_for_variant_id(ctx, variant.id())
        .await
        .expect("get prototypes")
        .into_iter()
        .find(|proto| proto.name() == "Import small odd lego")
        .expect("could not find prototype");

    let execution_result = management_prototype
        .execute(ctx, small_odd_lego.id())
        .await
        .expect("should execute management prototype func");

    let result: ManagementFuncReturn = execution_result
        .result
        .expect("should have a result success")
        .try_into()
        .expect("should be a valid management func return");

    assert_eq!(result.status, ManagementFuncStatus::Ok);

    let operations = result.operations.expect("should have operations");

    ManagementOperator::new(
        ctx,
        small_odd_lego.id(),
        execution_result.manager_component_geometry,
        operations,
        execution_result.managed_schema_map,
        execution_result.placeholders,
    )
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
