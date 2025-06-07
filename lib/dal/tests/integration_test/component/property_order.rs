use dal::{
    AttributeValue,
    DalContext,
    diagram::view::View,
    prop::Prop,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    expected::{
        ExpectComponent,
        ExpectComponentProp,
        ExpectSchemaVariant,
        IntoPropPath,
    },
    helpers::{
        ChangeSetTestHelpers,
        component,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

async fn prop_names(
    ctx: &DalContext,
    variant: ExpectSchemaVariant,
    prop_path: impl IntoPropPath,
) -> dal_test::Result<Vec<String>> {
    let prop = variant.prop(ctx, prop_path).await;
    Ok(Prop::direct_child_props_ordered(ctx, prop.id())
        .await?
        .into_iter()
        .map(|prop| prop.name)
        .collect())
}

async fn attribute_value_names(
    ctx: &DalContext,
    prop: ExpectComponentProp,
) -> dal_test::Result<Vec<String>> {
    let av_id = prop.attribute_value(ctx).await.id();
    let mut result = vec![];
    for child_av_id in AttributeValue::get_child_av_ids_in_order(ctx, av_id).await? {
        let prop = AttributeValue::prop(ctx, child_av_id).await?;
        result.push(prop.name);
    }
    Ok(result)
}

async fn create_abcdef_schema(ctx: &mut DalContext) -> dal_test::Result<ExpectSchemaVariant> {
    Ok(VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        "abcdef",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
        r#"
            function main() {
                return new AssetBuilder()
                    .addProp(new PropBuilder().setName("a").setKind("string").build())
                    .addProp(new PropBuilder().setName("b").setKind("string").build())
                    .addProp(new PropBuilder().setName("c").setKind("string").build())
                    .addProp(new PropBuilder().setName("d").setKind("string").build())
                    .addProp(new PropBuilder().setName("e").setKind("string").build())
                    .addProp(new PropBuilder().setName("f").setKind("string").build())
                    .build()
            }
        "#,
    )
    .await?
    .into())
}

async fn create_fedcba_schema(ctx: &mut DalContext) -> dal_test::Result<ExpectSchemaVariant> {
    Ok(VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        "fedcba",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
        r#"
            function main() {
                return new AssetBuilder()
                    .addProp(new PropBuilder().setName("f").setKind("string").build())
                    .addProp(new PropBuilder().setName("e").setKind("string").build())
                    .addProp(new PropBuilder().setName("d").setKind("string").build())
                    .addProp(new PropBuilder().setName("c").setKind("string").build())
                    .addProp(new PropBuilder().setName("b").setKind("string").build())
                    .addProp(new PropBuilder().setName("a").setKind("string").build())
                    .build()
            }
        "#,
    )
    .await?
    .into())
}

async fn create_test_object_prop_order_schema(
    ctx: &mut DalContext,
) -> dal_test::Result<ExpectSchemaVariant> {
    Ok(VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        "test_object_prop_order",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
        r#"
            function main() {
                return new AssetBuilder()
                    .addProp(new PropBuilder().setName("abcdef").setKind("object")
                        .addChild(new PropBuilder().setName("a").setKind("string").build())
                        .addChild(new PropBuilder().setName("b").setKind("string").build())
                        .addChild(new PropBuilder().setName("c").setKind("string").build())
                        .addChild(new PropBuilder().setName("d").setKind("string").build())
                        .addChild(new PropBuilder().setName("e").setKind("string").build())
                        .addChild(new PropBuilder().setName("f").setKind("string").build())
                        .build()
                    )
                    .addProp(new PropBuilder().setName("fedcba").setKind("object")
                        .addChild(new PropBuilder().setName("f").setKind("string").build())
                        .addChild(new PropBuilder().setName("e").setKind("string").build())
                        .addChild(new PropBuilder().setName("d").setKind("string").build())
                        .addChild(new PropBuilder().setName("c").setKind("string").build())
                        .addChild(new PropBuilder().setName("b").setKind("string").build())
                        .addChild(new PropBuilder().setName("a").setKind("string").build())
                        .build()
                    )
                    .build()
            }
        "#,
    )
    .await?
    .into())
}

const EXPECT_ABCDEF: [&str; 6] = ["a", "b", "c", "d", "e", "f"];
const EXPECT_FEDCBA: [&str; 6] = ["f", "e", "d", "c", "b", "a"];

#[test]
async fn property_order_remains_after_update(ctx: &mut DalContext) -> dal_test::Result<()> {
    // Validate that props are in schema defined order
    let abcdef_schema = create_abcdef_schema(ctx).await?;
    let fedcba_schema = create_fedcba_schema(ctx).await?;
    assert_eq!(
        prop_names(ctx, abcdef_schema, ["root", "domain"]).await?,
        EXPECT_ABCDEF
    );
    assert_eq!(
        prop_names(ctx, fedcba_schema, ["root", "domain"]).await?,
        EXPECT_FEDCBA,
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Create components and verify they retain order on start
    let abcdef_component = ExpectComponent(component::create(ctx, abcdef_schema, "abcdef").await?);
    let fedcba_component = ExpectComponent(component::create(ctx, fedcba_schema, "fedcba").await?);

    let abcdef = abcdef_component.prop(ctx, ["root", "domain"]).await;
    let fedcba = fedcba_component.prop(ctx, ["root", "domain"]).await;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Update both object prop values to {} and check if they stay in the same order
    AttributeValue::update(ctx, abcdef.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    AttributeValue::update(ctx, fedcba.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Ensure they stay in the same order after commit
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    Ok(())
}

#[test]
async fn child_property_order_remains_after_update(ctx: &mut DalContext) -> dal_test::Result<()> {
    // Validate that props are in schema defined order
    let variant = create_test_object_prop_order_schema(ctx).await?;
    assert_eq!(
        prop_names(ctx, variant, ["root", "domain", "abcdef"]).await?,
        EXPECT_ABCDEF,
    );
    assert_eq!(
        prop_names(ctx, variant, ["root", "domain", "fedcba"]).await?,
        EXPECT_FEDCBA,
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Create component and verify props retain order on start
    let component = variant.create_component_on_default_view(ctx).await;

    let abcdef = component.prop(ctx, ["root", "domain", "abcdef"]).await;
    let fedcba = component.prop(ctx, ["root", "domain", "fedcba"]).await;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Update both object prop values to {} and check if they stay in the same order
    AttributeValue::update(ctx, abcdef.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    AttributeValue::update(ctx, fedcba.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Ensure they stay in the same order after commit
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    Ok(())
}

#[test]
async fn child_property_order_remains_after_upgrade(ctx: &mut DalContext) -> dal_test::Result<()> {
    // Validate that props are in schema defined order
    let variant = create_test_object_prop_order_schema(ctx).await?;
    assert_eq!(
        prop_names(ctx, variant, ["root", "domain", "abcdef"]).await?,
        EXPECT_ABCDEF,
    );
    assert_eq!(
        prop_names(ctx, variant, ["root", "domain", "fedcba"]).await?,
        EXPECT_FEDCBA,
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Create component and verify props retain order on start
    let component = variant.create_component_on_default_view(ctx).await;

    let abcdef = component.prop(ctx, ["root", "domain", "abcdef"]).await;
    let fedcba = component.prop(ctx, ["root", "domain", "fedcba"]).await;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, &EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, &EXPECT_FEDCBA);

    // Update both object prop values to {} and check if they stay in the same order
    AttributeValue::update(ctx, abcdef.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    AttributeValue::update(ctx, fedcba.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Ensure they stay in the same order after commit
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Ensure they have the right order after upgrading
    let new_variant =
        ExpectSchemaVariant(VariantAuthoringClient::regenerate_variant(ctx, variant.id()).await?);
    let new_abcdef = component.prop(ctx, ["root", "domain", "abcdef"]).await;
    let new_fedcba = component.prop(ctx, ["root", "domain", "fedcba"]).await;
    assert_eq!(
        prop_names(ctx, new_variant, ["root", "domain", "abcdef"]).await?,
        EXPECT_ABCDEF,
    );
    assert_eq!(
        prop_names(ctx, new_variant, ["root", "domain", "fedcba"]).await?,
        EXPECT_FEDCBA,
    );

    assert_ne!(variant, new_variant);
    assert_eq!(component.schema_variant(ctx).await, new_variant);

    assert_eq!(attribute_value_names(ctx, new_abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, new_fedcba).await?, EXPECT_FEDCBA);

    Ok(())
}

#[test]
async fn child_property_value_remains_after_update_and_paste(
    ctx: &mut DalContext,
) -> dal_test::Result<()> {
    // Validate that props are in schema defined order
    let variant = create_test_object_prop_order_schema(ctx).await?;
    assert_eq!(
        prop_names(ctx, variant, ["root", "domain", "abcdef"]).await?,
        EXPECT_ABCDEF,
    );
    assert_eq!(
        prop_names(ctx, variant, ["root", "domain", "fedcba"]).await?,
        EXPECT_FEDCBA,
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Create component and verify props retain order on start
    let component = variant.create_component_on_default_view(ctx).await;

    let abcdef = component.prop(ctx, ["root", "domain", "abcdef"]).await;
    let fedcba = component.prop(ctx, ["root", "domain", "fedcba"]).await;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Update both object prop values to {} and check if they stay in the same order
    AttributeValue::update(ctx, abcdef.attribute_value(ctx).await.id(), Some(json!({}))).await?;
    AttributeValue::update(ctx, fedcba.attribute_value(ctx).await.id(), Some(json!({}))).await?;

    // Ensure they stay in the same order after commit
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);

    // Ensure they have the right values after updating them
    for name in ["a", "b", "c", "d", "e", "f"] {
        AttributeValue::update(
            ctx,
            component
                .prop(ctx, ["root", "domain", "abcdef", name])
                .await
                .attribute_value(ctx)
                .await
                .id(),
            Some(name.into()),
        )
        .await?;
        AttributeValue::update(
            ctx,
            component
                .prop(ctx, ["root", "domain", "fedcba", name])
                .await
                .attribute_value(ctx)
                .await
                .id(),
            Some(name.into()),
        )
        .await?;
    }
    assert_eq!(attribute_value_names(ctx, abcdef).await?, EXPECT_ABCDEF);
    assert_eq!(attribute_value_names(ctx, fedcba).await?, EXPECT_FEDCBA);
    assert_eq!(
        json!({"a": "a", "b": "b", "c": "c", "d": "d", "e": "e", "f": "f"}),
        abcdef.get(ctx).await
    );
    assert_eq!(
        json!({"f": "f", "e": "e", "d": "d", "c": "c", "b": "b", "a": "a"}),
        abcdef.get(ctx).await
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let default_view_id = View::get_id_for_default(ctx).await?;

    // Ensure they have the right values after copy/pasting
    let component_copy = ExpectComponent(
        component
            .component(ctx)
            .await
            .duplicate_without_connections(
                ctx,
                default_view_id,
                component.geometry_for_default(ctx).await,
            )
            .await
            .expect("unable to paste component")
            .id(),
    );
    let abcdef_copy = component_copy.prop(ctx, ["root", "domain", "abcdef"]).await;
    let fedcba_copy = component_copy.prop(ctx, ["root", "domain", "fedcba"]).await;
    assert_eq!(
        attribute_value_names(ctx, abcdef_copy).await?,
        EXPECT_ABCDEF,
    );
    assert_eq!(
        attribute_value_names(ctx, fedcba_copy).await?,
        EXPECT_FEDCBA,
    );
    assert_eq!(
        json!({"a": "a", "b": "b", "c": "c", "d": "d", "e": "e", "f": "f"}),
        abcdef_copy.get(ctx).await
    );
    assert_eq!(
        json!({"f": "f", "e": "e", "d": "d", "c": "c", "b": "b", "a": "a"}),
        fedcba_copy.get(ctx).await
    );

    Ok(())
}
