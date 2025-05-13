use dal::{
    AttributeValue,
    Component,
    DalContext,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        component,
        make_subscription,
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
    AttributeValue::subscribe(
        ctx,
        value_av_id,
        make_subscription(ctx, component_id, "/si/name").await?,
    )
    .await?;
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
    setup(ctx).await?;

    // Create a component with a Value prop
    let component_id = component::create(ctx, "testy", "testy").await?;
    let value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    // Create another component
    let other_component_id = component::create(ctx, "testy", "other").await?;
    let other_value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    AttributeValue::update(ctx, other_value_av_id, Some(json!("value"))).await?;

    // Subscribe to the value and see if it flows through!
    AttributeValue::subscribe(
        ctx,
        value_av_id,
        make_subscription(ctx, other_component_id, "/domain/Value").await?,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("value")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Update the value and watch the new value flow through!
    AttributeValue::update(ctx, other_value_av_id, Some(json!("value_2"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("value_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Unset the value and watch it flow through!
    AttributeValue::update(ctx, other_value_av_id, None).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);

    Ok(())
}

// AV subscribes to array element of another AV
#[test]
async fn subscribe_to_array_element(ctx: &mut DalContext) -> Result<()> {
    setup(ctx).await?;

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
    AttributeValue::subscribe(
        ctx,
        value_av_id,
        make_subscription(ctx, component_id, "/domain/Values/1").await?,
    )
    .await?;
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
    setup(ctx).await?;

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
    AttributeValue::subscribe(
        ctx,
        value_av_id,
        make_subscription(ctx, component_id, "/domain/ValueMap/B").await?,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("b")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );

    // Update the map value and watch the new value come through!
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
    setup(ctx).await?;

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
    assert_eq!(None, AttributeValue::view_by_id(ctx, value2_av_id).await?);

    // Create another component
    let other_component_id = component::create(ctx, "testy", "other").await?;
    let other_value_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
            .await?;
    let other_value2_av_id =
        Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value2"])
            .await?;
    AttributeValue::update(ctx, other_value_av_id, Some(json!("value"))).await?;
    AttributeValue::update(ctx, other_value2_av_id, Some(json!("value2"))).await?;

    // Subscribe to the values and see if it flows through!
    AttributeValue::subscribe(
        ctx,
        value_av_id,
        make_subscription(ctx, other_component_id, "/domain/Value").await?,
    )
    .await?;
    AttributeValue::subscribe(
        ctx,
        value2_av_id,
        make_subscription(ctx, other_component_id, "/domain/Value2").await?,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("value")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );
    assert_eq!(
        Some(json!("value2")),
        AttributeValue::view_by_id(ctx, value2_av_id).await?
    );

    // Update the values and watch them flow through!
    AttributeValue::update(ctx, other_value_av_id, Some(json!("value_2"))).await?;
    AttributeValue::update(ctx, other_value2_av_id, Some(json!("value2_2"))).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(
        Some(json!("value_2")),
        AttributeValue::view_by_id(ctx, value_av_id).await?
    );
    assert_eq!(
        Some(json!("value2_2")),
        AttributeValue::view_by_id(ctx, value2_av_id).await?
    );

    // Unset the values and watch them flow through!
    AttributeValue::update(ctx, other_value_av_id, None).await?;
    AttributeValue::update(ctx, other_value2_av_id, None).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(None, AttributeValue::view_by_id(ctx, value_av_id).await?);
    assert_eq!(None, AttributeValue::view_by_id(ctx, value2_av_id).await?);

    Ok(())
}

#[test]
async fn delete_component_with_subscriptions_correction(ctx: &mut DalContext) -> Result<()> {
    setup(ctx).await?;

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
    AttributeValue::subscribe(
        ctx,
        value_av_id,
        make_subscription(ctx, other_component_id, "/domain/Value").await?,
    )
    .await?;

    let other_component_root_av_id =
        Component::root_attribute_value_id(ctx, other_component_id).await?;

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    let cs_1 = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    AttributeValue::subscribe(
        ctx,
        value2_av_id,
        make_subscription(ctx, other_component_id, "/domain/Value2").await?,
    )
    .await?;

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

async fn setup(ctx: &DalContext) -> Result<()> {
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
