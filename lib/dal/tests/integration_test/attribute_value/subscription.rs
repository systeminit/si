use dal::{
    AttributeValue,
    Component,
    DalContext,
    attribute::value::subscription::ValueSubscription,
    func::authoring::FuncAuthoringClient,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::{
            value,
            value::AttributeValueKey,
        },
        change_set,
        component,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

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
    let component_id = component::create(ctx, "testy", "my name is testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    // Subscribe to the value and see if it flows through!
    value::subscribe(ctx, value_av_id, [(component_id, "/si/name")]).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("my name is testy")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Update the name and watch the new value flow through!
    // Update the name and watch the new value flow through!
    Component::get_by_id(ctx, component_id)
        .await?
        .set_name(ctx, "testy_2")
        .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("testy_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Unset

    Ok(())
}

// AV subscribes to AV on same component
#[test]
async fn subscribe_to_string(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    change_set::commit(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    // Create another component
    let other_component_id = component::create(ctx, "testy", "other").await?;

    Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"]).await?;
    value::set(ctx, ("other", "/domain/Value"), "value").await?;

    // Subscribe to the value and see if it flows through!
    value::subscribe(ctx, value_av_id, [(other_component_id, "/domain/Value")]).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("value")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Update the value and watch the new value flow through!
    value::set(ctx, ("other", "/domain/Value"), "value_2").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("value_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Unset the value and watch it flow through!
    value::unset(ctx, ("other", "/domain/Value")).await?;
    change_set::commit(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    Ok(())
}

// AV subscribes to array element of another AV
#[test]
async fn subscribe_to_array_element(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let values_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Values"])
            .await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("a")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("b")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("c")), None).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    // Subscribe to a specific index and watch the value come through!
    value::subscribe(ctx, value_av_id, [(component_id, "/domain/Values/1")]).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("b")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Update the array and watch the new value come through!
    AttributeValue::update(ctx, values_av_id, Some(json!([]))).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("a_2")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("b_2")), None).await?;
    AttributeValue::insert(ctx, values_av_id, Some(json!("c_2")), None).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("b_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // // Update the array with fewer values and watch the value disappear!
    // AttributeValue::update(ctx, values_av_id, Some(json!(["a_3"]))).await?;
    // ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    Ok(())
}

// AV subscribes to map element of another AV
#[test]
async fn subscribe_to_map_element(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let value_map_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "ValueMap"])
            .await?;
    AttributeValue::insert(ctx, value_map_av_id, Some(json!("a")), Some("A".to_owned())).await?;
    AttributeValue::insert(ctx, value_map_av_id, Some(json!("b")), Some("B".to_owned())).await?;
    AttributeValue::insert(ctx, value_map_av_id, Some(json!("c")), Some("C".to_owned())).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    // Subscribe to a specific index and watch the value come through!
    value::subscribe(ctx, value_av_id, [(component_id, "/domain/ValueMap/B")]).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("b")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Update the map value and watch the new value come through!
    value::set(ctx, ("testy", "/domain/ValueMap/B"), "b_2").await?;
    AttributeValue::insert(
        ctx,
        value_map_av_id,
        Some(json!("b_2")),
        Some("B".to_owned()),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("b_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // // Remove the map value and watch the value disappear!
    // AttributeValue::insert(ctx, value_map_av_id, None, Some("B".to_owned())).await?;
    // ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    Ok(())
}

// Two different subscriptions to different values on the same component (tests dirty logic)
#[test]
async fn subscribe_to_two_values(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let value2_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value2"])
            .await?;
    change_set::commit(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);
    assert_eq!(None, AttributeValue::view_by_id(ctx, value2_av_id).await?);

    // Create another component
    let other_component_id = component::create(ctx, "testy", "other").await?;
    value::set(ctx, ("other", "/domain/Value"), "value").await?;
    value::set(ctx, ("other", "/domain/Value2"), "value2").await?;

    // Subscribe to the values and see if it flows through!
    value::subscribe(ctx, value_av_id, [(other_component_id, "/domain/Value")]).await?;
    value::subscribe(ctx, value2_av_id, [(other_component_id, "/domain/Value2")]).await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("value")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );
    assert_eq!(
        Some(json!("value2")),
        AttributeValue::view_by_id(ctx, value2_av_id).await?
    );

    // Update the values and watch them flow through!
    value::set(ctx, ("other", "/domain/Value"), "value_2").await?;
    value::set(ctx, ("other", "/domain/Value2"), "value2_2").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        Some(json!("value_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );
    assert_eq!(
        Some(json!("value2_2")),
        AttributeValue::view_by_id(ctx, value2_av_id).await?
    );

    // Unset the values and watch them flow through!
    value::unset(ctx, ("other", "/domain/Value")).await?;
    value::unset(ctx, ("other", "/domain/Value2")).await?;
    change_set::commit(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);
    assert_eq!(None, AttributeValue::view_by_id(ctx, value2_av_id).await?);

    Ok(())
}

#[test]
async fn delete_component_with_subscriptions_correction(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let value2_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value2"])
            .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    // Create another component
    let other_component_id = component::create(ctx, "testy", "other").await?;
    let other_value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    AttributeValue::update(ctx, other_value_av_id, Some(json!("value"))).await?;

    // Subscribe to the values and see if it flows through!
    value::subscribe(ctx, value_av_id, [(other_component_id, "/domain/Value")]).await?;

    let other_component_root_av_id =
        Component::root_attribute_value_id(ctx, other_component_id).await?;

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    let cs_1 = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    value::subscribe(ctx, value2_av_id, [(other_component_id, "/domain/Value2")]).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let _cs_2 = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    Component::remove(ctx, other_component_id).await?;

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    assert!(
        !ctx.workspace_snapshot()?
            .node_exists(other_component_id)
            .await
    );

    assert!(
        !ctx.workspace_snapshot()?
            .node_exists(other_component_root_av_id)
            .await
    );

    ctx.update_visibility_and_snapshot_to_visibility(cs_1.id)
        .await?;

    assert!(
        !ctx.workspace_snapshot()?
            .node_exists(other_component_id)
            .await
    );

    assert!(
        !ctx.workspace_snapshot()?
            .node_exists(other_component_root_av_id)
            .await
    );

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
        [("input", "/domain/Values")],
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
async fn array_single_subscription(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Create another component and subscribe to one of its values
    component::create(ctx, "testy", "input").await?;
    value::set(ctx, ("input", "/domain/Value"), "value1").await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Values"),
        [("input", "/domain/Value")],
    )
    .await?;
    change_set::commit(ctx).await?;
    // Make sure it was upleveled to an array
    assert_eq!(
        json!(["value1"]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    // Update the values and watch them flow through!
    value::set(ctx, ("input", "/domain/Value"), "alt1").await?;
    change_set::commit(ctx).await?;
    assert_eq!(
        json!(["alt1"]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    Ok(())
}

#[test]
async fn array_multiple_subscriptions(ctx: &mut DalContext) -> Result<()> {
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
        ("subscriber", "/domain/Values"),
        [("input", "/domain/Value"), ("input", "/domain/Value2")],
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
async fn array_zero_subscriptions(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Subscribe to nothing and make sure the value is an empty array
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Values"),
        Vec::<ValueSubscription>::new(),
    )
    .await?;
    change_set::commit(ctx).await?;
    // Make sure it was upleveled to an array
    assert_eq!(
        json!([]),
        value::get(ctx, ("subscriber", "/domain/Values")).await?
    );

    Ok(())
}

#[test]
async fn subscription_count_errors(ctx: &mut DalContext) -> Result<()> {
    create_testy_variant(ctx).await?;

    // Create a component with a Value prop
    component::create(ctx, "testy", "subscriber").await?;
    change_set::commit(ctx).await?;
    assert!(!value::has_value(ctx, ("subscriber", "/domain/Values")).await?);

    // Create another component and subscribe to its values
    component::create(ctx, "testy", "input").await?;

    // Test that you cannot subscribe a single valued prop to multiple subscriptions
    assert!(
        value::subscribe(
            ctx,
            ("subscriber", "/domain/Value"),
            [("input", "/domain/Value"), ("input", "/domain/Value2")],
        )
        .await
        .is_err()
    );

    // Test that you cannot subscribe a single valued prop to zero subscriptions
    assert!(
        value::subscribe(
            ctx,
            ("subscriber", "/domain/Value"),
            Vec::<ValueSubscription>::new(),
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
        [("input", "/domain/Values/0")],
    )
    .await?;
    value::subscribe(
        ctx,
        ("subscriber", "/domain/Value2"),
        [("input", "/domain/Values/1")],
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
    AttributeValue::remove_by_id(
        ctx,
        ("input", "/domain/Values/0")
            .lookup_attribute_value(ctx)
            .await?,
    )
    .await?;
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
        [("source", "/domain/Value")],
        Some(func.id),
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
                        { name: "Values", kind: "array",
                            entry: { name: "ValuesItem", kind: "string" },
                        },
                        { name: "ValueMap", kind: "map",
                            entry: { name: "ValueMapItem", kind: "string" },
                        },
                    ]
                };
            }
        "#,
    )
    .await?;

    Ok(())
}
