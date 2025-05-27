use dal::{
    AttributeValue,
    Component,
    DalContext,
};
use dal_test::{
    Result,
    expected::ExpectComponent,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value::{
            self,
            AttributeValueKey,
        },
        change_set,
        component,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

mod subscription;

#[test]
async fn arguments_for_prototype_function_execution(ctx: &mut DalContext) -> Result<()> {
    // Create a component and commit. For context, the test exclusive schema has the identity
    // function set on "/root/domain/name" with an input from "/root/si/name". We need to ensure
    // that the value of "/root/si/name" comes in, as expected. The name is set when creating a
    // component, so we do not need to do additional setup.
    let expected = "you should see this name in the arguments";
    let component = ExpectComponent::create_named(ctx, "swifty", expected).await;
    let name_prop = component.prop(ctx, ["root", "domain", "name"]).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Ensure that the arguments look as we expect.
    let name_av_id = name_prop.attribute_value(ctx).await.id();
    let (_, arguments, _) =
        AttributeValue::prepare_arguments_for_prototype_function_execution(ctx, name_av_id).await?;
    assert_eq!(
        json![{
            "identity": expected
        }],
        arguments
    );
    Ok(())
}

#[test]
async fn attribute_value_path(ctx: &mut DalContext) -> Result<()> {
    // Create a component and commit. For context, the test exclusive schema has the identity
    // function set on "/root/domain/name" with an input from "/root/si/name". We need to ensure
    // that the value of "/root/si/name" comes in, as expected. The name is set when creating a
    // component, so we do not need to do additional setup.
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Value", kind: "string" },
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
    let component_id = component::create(ctx, "test", "test").await?;

    // Check object paths
    {
        let value =
            Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Value"])
                .await?;
        AttributeValue::update(ctx, value, Some(json!("test"))).await?;
        assert_eq!(
            AttributeValue::path_from_root(ctx, value).await?.1,
            "/domain/Value"
        );
    }

    // Check array paths
    {
        let values =
            Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "Values"])
                .await?;
        AttributeValue::update(ctx, values, Some(json!(["1", "2", "3"]))).await?;
        let values_elements = AttributeValue::get_child_av_ids_in_order(ctx, values).await?;

        assert_eq!(
            AttributeValue::path_from_root(ctx, values).await?.1,
            "/domain/Values"
        );
        assert_eq!(
            AttributeValue::path_from_root(ctx, values_elements[0])
                .await?
                .1,
            "/domain/Values/0"
        );
    }

    // Check map paths
    {
        let value_map =
            Component::attribute_value_for_prop(ctx, component_id, &["root", "domain", "ValueMap"])
                .await?;
        AttributeValue::update(ctx, value_map, Some(json!({ "a": "1", "b": "2", "c": "3"})))
            .await?;
        let value_map_elements = AttributeValue::map_children(ctx, value_map).await?;
        assert_eq!(
            AttributeValue::path_from_root(ctx, value_map).await?.1,
            "/domain/ValueMap"
        );
        assert_eq!(
            AttributeValue::path_from_root(ctx, *value_map_elements.get("a").unwrap())
                .await?
                .1,
            "/domain/ValueMap/a"
        );
    }

    Ok(())
}

#[test]
async fn update_object_multiplayer(ctx: &mut DalContext) -> Result<()> {
    // Create a variant that will be in all three change sets
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "ObjectValue", kind: "object", children: [
                            { name: "a", kind: "string" },
                            { name: "b", kind: "string" },
                            { name: "both", kind: "string" },
                            { name: "neither", kind: "string" },
                        ] },
                        { name: "ArrayOfObjectValues", kind: "array",
                            entry: { name: "ArrayOfObjectValuesItem", kind: "object", children: [
                                { name: "a", kind: "string" },
                                { name: "b", kind: "string" },
                                { name: "both", kind: "string" },
                                { name: "neither", kind: "string" },
                            ] },
                        },
                        { name: "MapOfObjectValues", kind: "map",
                            entry: { name: "MapOfObjectValuesItem", kind: "object", children: [
                                { name: "a", kind: "string" },
                                { name: "b", kind: "string" },
                                { name: "both", kind: "string" },
                                { name: "neither", kind: "string" },
                            ] },
                        },
                    ]
                };
            }
        "#,
    )
    .await?;
    component::create(ctx, "test", "test").await?;
    // Give them all initial values (testing the case where the value is already set, and changesets update)
    let attrs = [
        ("test", "/domain/ObjectValue"),
        ("test", "/domain/ArrayOfObjectValues/0"),
        ("test", "/domain/MapOfObjectValues/val"),
    ];
    for &attr in &attrs {
        value::set(ctx, attr, json!({})).await?;
    }
    change_set::commit(ctx).await?;

    // Two players update the object in parallel.
    let (player1, player2) = (ctx.clone(), ctx.clone());
    {
        for &attr in &attrs {
            value::set(&player1, attr, json!({ "a": "a", "both": "a"})).await?;
        }
        player1.commit().await?;
    }
    {
        for &attr in &attrs {
            value::set(&player2, attr, json!({ "b": "b", "both": "b"})).await?;
        }
        player2.commit().await?;
    }

    // Check its effect on the merged changeset
    ChangeSetTestHelpers::wait_for_dvu(ctx).await?;
    ctx.update_snapshot_to_visibility().await?;
    for &attr in &attrs {
        // The object must have exactly 4 child AVs and the value should be the merged value
        let av_id = attr.lookup_attribute_value(ctx).await?;
        assert_eq!(4, AttributeValue::child_av_ids(ctx, av_id).await?.len(),);
        assert_eq!(
            json!({ "b": "b", "both": "b", "a": "a" }),
            value::get(ctx, attr).await?,
        );
    }

    Ok(())
}
