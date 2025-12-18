use std::collections::BTreeSet;

use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    component::subscription_graph::SubscriptionGraph,
    func::authoring::FuncAuthoringClient,
    workspace_snapshot::graph::validator::ValidationIssue,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        change_set,
        component,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use si_id::ComponentId;

pub mod default_subscriptions;

// AV subscribes to name AV on same component
#[test]
async fn subscribe_to_name_on_same_component(ctx: &mut DalContext) -> Result<()> {
    // Make a variant with a Value prop
    variant::create(
        ctx,
        "testy",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Value", kind: "string" },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Create a component with a Value prop and a name
    let component_id = component::create(ctx, "testy", "initial name").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, (component_id, "/domain/Value")).await?);

    // Subscribe to the value and see if it flows through!
    value::subscribe(
        ctx,
        (component_id, "/domain/Value"),
        (component_id, "/si/name"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("initial name"),
        value::get(ctx, (component_id, "/domain/Value")).await?
    );

    // Update the name and watch the new value flow through!
    value::set(ctx, (component_id, "/si/name"), "updated name").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("updated name"),
        value::get(ctx, (component_id, "/domain/Value")).await?
    );

    Ok(())
}

// AV subscribes to AV on same component
#[test]
async fn subscribe_to_string(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);

    // Create another component
    component::create(ctx, "testy", "source").await?;
    value::set(ctx, ("source", "/domain/Value"), "value").await?;

    // Subscribe to the value and see if it flows through!
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("source", "/domain/Value"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("value"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );

    // Update the value and watch the new value flow through!
    value::set(ctx, ("source", "/domain/Value"), "value_2").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("value_2"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );

    // Unset the value and watch it flow through!
    value::unset(ctx, ("source", "/domain/Value")).await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);

    Ok(())
}

// AV subscribes to array element of another AV
#[test]
async fn subscribe_to_array_element(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "subscriber").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let values_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Values"])
            .await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("a")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("b")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("c")), None).await?;
    change_set::commit(ctx).await?;
    assert_eq!(None, AttributeValue::view(ctx, value_av_id).await?);

    // Subscribe to a specific index and watch the value come through!
    value::subscribe(ctx, value_av_id, (component_id, "/domain/Values/1")).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("b")),
        AttributeValue::view(ctx, value_av_id).await?
    );

    // Update the array and watch the new value come through!
    AttributeValue::update(ctx, values_av_id, Some(json!([]))).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("a_2")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("b_2")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("c_2")), None).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("b_2")),
        AttributeValue::view(ctx, value_av_id).await?
    );

    // // Update the array with fewer values and watch the value disappear!
    // AttributeValue::update(ctx, values_av_id, Some(json!(["a_3"]))).await?;
    // change_set::commit(ctx).await?;
    // assert_eq!(None, AttributeValue::view(ctx, value_av_id).await?);

    Ok(())
}

// AV subscribes to map element of another AV
#[test]
async fn subscribe_to_map_element(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "subscriber").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let value_map_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "ValueMap"])
            .await?;
    AttributeValue::insert(ctx, value_map_av_id, Some(json!("a")), Some("A".to_owned())).await?;
    AttributeValue::insert(ctx, value_map_av_id, Some(json!("b")), Some("B".to_owned())).await?;
    AttributeValue::insert(ctx, value_map_av_id, Some(json!("c")), Some("C".to_owned())).await?;
    change_set::commit(ctx).await?;
    assert_eq!(None, AttributeValue::view(ctx, value_av_id).await?);

    // Subscribe to a specific index and watch the value come through!
    value::subscribe(ctx, value_av_id, (component_id, "/domain/ValueMap/B")).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("b")),
        AttributeValue::view(ctx, value_av_id).await?
    );

    // Update the map value and watch the new value come through!
    value::set(ctx, ("subscriber", "/domain/ValueMap/B"), "b_2").await?;
    AttributeValue::insert(
        ctx,
        value_map_av_id,
        Some(json!("b_2")),
        Some("B".to_owned()),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("b_2")),
        AttributeValue::view(ctx, value_av_id).await?
    );

    // // Remove the map value and watch the value disappear!
    // AttributeValue::insert(ctx, value_map_av_id, None, Some("B".to_owned())).await?;
    // change_set::commit(ctx).await?;
    // assert_eq!(None, AttributeValue::view(ctx, value_av_id).await?);

    Ok(())
}

// Two different subscriptions to different values on the same component (tests dirty logic)
#[test]
async fn subscribe_to_two_values(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value2")).await?);

    // Create another component
    component::create(ctx, "testy", "source").await?;
    value::set(ctx, ("source", "/domain/Value"), "value").await?;
    value::set(ctx, ("source", "/domain/Value2"), "value2").await?;

    // Subscribe to the values and see if it flows through!
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("source", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value2"),
        ("source", "/domain/Value2"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("value"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );
    assert_eq!(
        json!("value2"),
        value::get(ctx, ("subscriber", "/domain/Value2")).await?
    );

    // Update the values and watch them flow through!
    value::set(ctx, ("source", "/domain/Value"), "value_2").await?;
    value::set(ctx, ("source", "/domain/Value2"), "value2_2").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("value_2"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );
    assert_eq!(
        json!("value2_2"),
        value::get(ctx, ("subscriber", "/domain/Value2")).await?
    );

    // Unset the values and watch them flow through!
    value::unset(ctx, ("source", "/domain/Value")).await?;
    value::unset(ctx, ("source", "/domain/Value2")).await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value2")).await?);

    Ok(())
}

#[test]
async fn delete_component_with_subscriptions_correction(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;

    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);

    // Create another component
    let source_id = component::create(ctx, "testy", "source").await?;
    value::set(ctx, ("subscriber", "/domain/Value"), "value").await?;

    // Subscribe to the values and see if it flows through!
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("source", "/domain/Value"),
    )
    .await?;

    let source_root_id = Component::root_attribute_value_id(ctx, source_id).await?;

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    let cs_1 = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value2"),
        ("source", "/domain/Value2"),
    )
    .await?;

    change_set::commit(ctx).await?;

    let _cs_2 = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    Component::remove(ctx, source_id).await?;

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    assert!(!ctx.workspace_snapshot()?.node_exists(source_id).await);

    assert!(!ctx.workspace_snapshot()?.node_exists(source_root_id).await);

    ctx.update_visibility_and_snapshot_to_visibility(cs_1.id)
        .await?;

    assert!(!ctx.workspace_snapshot()?.node_exists(source_id).await);

    assert!(!ctx.workspace_snapshot()?.node_exists(source_root_id).await);

    Ok(())
}

#[test]
async fn array_subscription(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Create another component and subscribe to its values
    component::create(ctx, "testy", "input").await?;
    value::set(ctx, ("input", "/domain/Values"), ["value1", "value2"]).await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Values"),
        ("input", "/domain/Values"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!(["value1", "value2"]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    // Update the values and watch them flow through!
    value::set(ctx, ("input", "/domain/Values"), ["alt1", "alt2"]).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!(["alt1", "alt2"]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    Ok(())
}

#[test]
async fn array_item_subscriptions(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Create another component and subscribe to its values
    component::create(ctx, "testy", "input").await?;
    value::set(ctx, ("input", "/domain/Value"), "value1").await?;
    value::set(ctx, ("input", "/domain/Value2"), "value2").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Values/-"),
        ("input", "/domain/Value"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Values/-"),
        ("input", "/domain/Value2"),
    )
    .await?;
    change_set::commit(ctx).await?;
    // Make sure they were upleveled to an array
    assert_eq!(
        json!(["value1", "value2"]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    // Update the values and watch them flow through!
    value::set(ctx, ("input", "/domain/Value"), "alt1").await?;
    value::set(ctx, ("input", "/domain/Value2"), "alt2").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!(["alt1", "alt2"]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    Ok(())
}

#[test]
async fn subscription_object_to_map(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create and subscribe subscriber:ObjectValue -> input:ValueMap
    component::create(ctx, "testy", "input").await?;
    component::create(ctx, "testy", "subscriber").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/ObjectValue"),
        ("input", "/domain/ValueMap"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(json!({}), component::domain(ctx, "subscriber").await?);

    // Set input:ValueMap and watch the value flow through to subscriber
    value::set(
        ctx,
        ("input", "/domain/ValueMap"),
        json!({ "Foo": "foo", "Bar": "bar", "Baz": "baz" }),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({ "ObjectValue": { "Foo": "foo", "Bar": "bar" } }),
        component::domain(ctx, "subscriber").await?
    );

    Ok(())
}

#[test]
async fn subscription_map_to_object(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create and subscribe subscriber:ValueMap -> input:ObjectValue
    component::create(ctx, "testy", "input").await?;
    component::create(ctx, "testy", "subscriber").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/ValueMap"),
        ("input", "/domain/ObjectValue"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(json!({}), component::domain(ctx, "subscriber").await?);

    // Set input:ObjectValue and watch the value flow through to subscriber
    value::set(
        ctx,
        ("input", "/domain/ObjectValue"),
        json!({ "Foo": "foo", "Bar": "bar", "Baz": "baz" }),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({ "ValueMap": { "Foo": "foo", "Bar": "bar" } }),
        component::domain(ctx, "subscriber").await?
    );

    Ok(())
}

#[test]
async fn subscription_json_to_string(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create and subscribe subscriber:JsonValue -> input:Value
    component::create(ctx, "testy", "input").await?;
    component::create(ctx, "testy", "subscriber").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/JsonValue"),
        ("input", "/domain/Value"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(json!({}), component::domain(ctx, "subscriber").await?);

    // Set input:Value and watch the value flow through to subscriber
    value::set(ctx, ("input", "/domain/Value"), "value").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({ "JsonValue": "value" }),
        component::domain(ctx, "subscriber").await?
    );

    Ok(())
}

#[test]
async fn subscription_string_to_json(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create and subscribe subscriber:Value -> input:JsonValue
    component::create(ctx, "testy", "input").await?;
    component::create(ctx, "testy", "subscriber").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("input", "/domain/JsonValue"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(json!({}), component::domain(ctx, "subscriber").await?);

    // Set input:JsonValue and watch the value flow through to subscriber
    value::set(ctx, ("input", "/domain/JsonValue"), "value").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!({ "Value": "value" }),
        component::domain(ctx, "subscriber").await?
    );

    Ok(())
}

#[test]
async fn subscription_type_mismatch_array_to_single(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Create another component and subscribe to one of its values
    component::create(ctx, "testy", "input").await?;
    value::set(ctx, ("input", "/domain/Value"), "value1").await?;
    assert!(
        value::subscribe(
            ctx,
            ("subscriber", "/domain/Values"),
            ("input", "/domain/Value"),
        )
        .await
        .is_err()
    );

    Ok(())
}

#[test]
async fn subscription_type_mismatch_single_to_array(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Create another component and subscribe to one of its values
    component::create(ctx, "testy", "input").await?;
    value::set(ctx, ("input", "/domain/Value"), "value1").await?;
    assert!(
        value::subscribe(
            ctx,
            ("subscriber", "/domain/Value"),
            ("input", "/domain/Values"),
        )
        .await
        .is_err()
    );

    Ok(())
}

#[test]
async fn delete_subscribed_to_array_item(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;

    // Create another component and subscribe to its values
    component::create(ctx, "testy", "input").await?;

    // Subscribe value props to first and second array items
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("input", "/domain/Values/0"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value2"),
        ("input", "/domain/Values/1"),
    )
    .await?;

    // Create 3 array items on input
    value::set(ctx, ("input", "/domain/Values/0"), "test_value_0").await?;
    value::set(ctx, ("input", "/domain/Values/1"), "test_value_1").await?;

    change_set::commit(ctx).await?;

    // Make sure subscriptions worked!
    assert_eq!(
        json!("test_value_0"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );
    assert_eq!(
        json!("test_value_1"),
        value::get(ctx, ("subscriber", "/domain/Value2")).await?
    );

    // Delete source array item and validated that the null value has been propagated
    AttributeValue::remove(ctx, value::id(ctx, ("input", "/domain/Values/0")).await?).await?;
    change_set::commit(ctx).await?;

    // Make sure that the DVU set the right new values to the subscriber props
    assert_eq!(
        json!("test_value_1"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value2")).await?);

    Ok(())
}

#[test]
async fn subscribe_with_custom_function(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;
    let func = FuncAuthoringClient::create_new_transformation_func(
        ctx,
        Some("make_it_better".to_string()),
    )
    .await?;

    FuncAuthoringClient::save_code(
        ctx,
        func.id,
        "function main({input}) {return `${ input }, but better`;}",
    )
    .await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;

    // Create another component and subscribe to its values
    component::create(ctx, "testy", "source").await?;

    // Subscribe Value
    value::subscribe_with_custom_function(
        ctx,
        ("subscriber", "/domain/Value"),
        ("source", "/domain/Value"),
        func.id,
    )
    .await?;

    change_set::commit(ctx).await?;

    // Make sure transformation works with initial unset value as source
    assert_eq!(
        json!("null, but better"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );

    // now set a value on source
    value::set(ctx, ("source", "/domain/Value"), "test value").await?;

    change_set::commit(ctx).await?;

    // Make sure value setting and subscription worked!
    assert_eq!(
        json!("test value"),
        value::get(ctx, ("source", "/domain/Value")).await?
    );

    assert_eq!(
        json!("test value, but better"),
        value::get(ctx, ("subscriber", "/domain/Value")).await?
    );

    Ok(())
}

#[test]
async fn remove_subscribed_component(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);

    // Create another component and subscribe to its values
    let source_id = component::create(ctx, "testy", "source").await?;
    value::set(ctx, ("source", "/domain/Value"), "value").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("source", "/domain/Value"),
    )
    .await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!("value"),
        value::get(ctx, ("subscriber", "/domain/Value"))
            .await
            .expect("value should exist")
    );

    // Remove the source component and make sure the subscriber value is unset
    Component::remove(ctx, source_id).await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Value")).await?);

    // Make sure the graph looks like what we want: the subscriber has a prototype with zero
    // arguments.
    let av_id = value::id(ctx, ("subscriber", "/domain/Value")).await?;
    let prototype_id = AttributeValue::component_prototype_id(ctx, av_id)
        .await?
        .expect("should still have a prototype after subscription is removed");
    assert!(
        AttributePrototype::list_arguments(ctx, prototype_id)
            .await?
            .is_empty()
    );

    Ok(())
}

#[test]
async fn subscription_cycles(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;
    component::create(ctx, "testy", "subscriber").await?;
    component::create(ctx, "testy", "source").await?;
    component::create(ctx, "testy", "source_2").await?;

    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value"),
        ("source", "/domain/Value"),
    )
    .await?;

    value::subscribe(
        ctx,
        ("source", "/domain/Value"),
        ("source_2", "/domain/Value"),
    )
    .await?;

    // Self subscription should be fine
    value::subscribe(
        ctx,
        ("source_2", "/domain/Value3"),
        ("source_2", "/domain/Value4"),
    )
    .await?;

    ctx.workspace_snapshot()?.cleanup().await?;

    let issues = ctx
        .workspace_snapshot()?
        .as_legacy_snapshot()?
        .validate()
        .await?;

    assert!(issues.is_empty());

    value::subscribe(
        ctx,
        ("source_2", "/domain/Value2"),
        ("subscriber", "/domain/Value2"),
    )
    .await?;

    let issues = ctx
        .workspace_snapshot()?
        .as_legacy_snapshot()?
        .validate()
        .await?;

    assert_eq!(
        &[(
            ValidationIssue::CyclicSubscriptions,
            "Cycle in the subscriptions between components".to_string(),
        ),],
        issues.as_slice()
    );

    Ok(())
}

async fn create_testy_variant(ctx: &DalContext) -> Result<()> {
    // Make a variant with a Value prop
    variant::create(
        ctx,
        "testy",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Value", kind: "string" },
                        { name: "Value2", kind: "string" },
                        { name: "Value3", kind: "string" },
                        { name: "Value4", kind: "string" },
                        { name: "Values", kind: "array",
                            entry: { name: "ValuesItem", kind: "string" },
                        },
                        { name: "ValueMap", kind: "map",
                            entry: { name: "ValueMapItem", kind: "string" },
                        },
                        { name: "JsonValue", kind: "json" },
                        { name: "ObjectValue", kind: "object",
                            children: [
                                { name: "Foo", kind: "string" },
                                { name: "Bar", kind: "string" },
                            ]
                        },
                    ]
                };
            }
        "#,
    )
    .await?;

    Ok(())
}

/// Helper to extract the set of (subscriber, source) edges from a SubscriptionGraph
fn extract_edges(graph: &SubscriptionGraph) -> BTreeSet<(ComponentId, ComponentId)> {
    let inner = graph.inner();
    let mut edges = BTreeSet::new();

    for id in inner.all_ids() {
        for dep in inner.direct_dependencies_of(id) {
            edges.insert((id, dep));
        }
    }

    edges
}

#[test]
async fn subscription_graph_constructors_produce_identical_results(
    ctx: &mut DalContext,
) -> Result<()> {
    create_testy_variant(ctx).await?;

    component::create(ctx, "testy", "subscriber1").await?;
    component::create(ctx, "testy", "subscriber2").await?;
    component::create(ctx, "testy", "source1").await?;
    component::create(ctx, "testy", "source2").await?;

    value::set(ctx, ("source1", "/domain/Value"), "value1").await?;
    value::set(ctx, ("source2", "/domain/Value"), "value2").await?;

    value::subscribe(
        ctx,
        ("subscriber1", "/domain/Value"),
        ("source1", "/domain/Value"),
    )
    .await?;

    value::subscribe(
        ctx,
        ("subscriber2", "/domain/Value"),
        ("source1", "/domain/Value2"),
    )
    .await?;

    value::subscribe(
        ctx,
        ("subscriber2", "/domain/Value2"),
        ("source2", "/domain/Value"),
    )
    .await?;

    let sub_graph_from_dal_ctx = SubscriptionGraph::new(ctx).await?;
    let snapshot = ctx.workspace_snapshot()?.as_legacy_snapshot()?;
    let working_copy = snapshot.working_copy_cloned().await;
    let sub_graph_from_snapshot = SubscriptionGraph::new_from_snapshot(&working_copy)?;

    let dal_edges = extract_edges(&sub_graph_from_dal_ctx);
    let snapshot_edges = extract_edges(&sub_graph_from_snapshot);

    assert!(
        !dal_edges.is_empty(),
        "There should be at least some subscription edges"
    );

    assert_eq!(
        dal_edges, snapshot_edges,
        "Both constructors should produce identical subscription graphs"
    );

    assert_eq!(
        sub_graph_from_dal_ctx.is_cyclic(),
        sub_graph_from_snapshot.is_cyclic(),
        "Both graphs should agree on whether the graph is cyclic"
    );

    Ok(())
}
