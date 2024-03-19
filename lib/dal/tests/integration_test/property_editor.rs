use dal::property_editor::schema::PropertyEditorSchema;
use dal::property_editor::values::PropertyEditorValues;
use dal::{AttributeValue, Component, DalContext, Schema, SchemaVariant};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;

#[test]
async fn assemble(ctx: &DalContext) {
    // List all schemas in the workspace. Pick the first one alphabetically.
    let mut schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    let schema = schemas.pop().expect("schemas are empty");

    // Pick a schema variant.
    let mut schema_variants = SchemaVariant::list_for_schema(ctx, schema.id())
        .await
        .expect("could not list schema variants for schema");
    let schema_variant = schema_variants.pop().expect("schemas are empty");
    let schema_variant_id = schema_variant.id();

    // Create a component and set geometry.
    let name = "steam deck";
    let component = Component::new(ctx, name, schema_variant_id)
        .await
        .expect("could not create component");

    // Assemble both property editor blobs.
    let property_editor_schema = PropertyEditorSchema::assemble(ctx, schema_variant_id)
        .await
        .expect("could not assemble property editor schema");
    let property_editor_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("could not assemble property editor schema");
    dbg!(property_editor_schema, property_editor_values);
}

#[test]
async fn array_map_manipulation(ctx: &DalContext) {
    // Create a component using the testing schema
    let component = create_component_for_schema_name(ctx, "pirate", "ss poopcanoe").await;
    // let variant_id = Component::schema_variant_id(ctx, component.id())
    //     .await
    //     .expect("find variant id for component");

    let parrot_names_value_id = component
        .attribute_values_for_prop(ctx, &["root", "domain", "parrot_names"])
        .await
        .expect("find value ids for the prop parrot_names")
        .pop()
        .expect("there should only be one value id");

    let treasure_map_value_id = component
        .attribute_values_for_prop(ctx, &["root", "domain", "treasure"])
        .await
        .expect("find value ids for the prop treasure")
        .pop()
        .expect("there should only be one value id");

    // Add items to array prop
    AttributeValue::insert(ctx, parrot_names_value_id, Some("tabitha".into()), None)
        .await
        .expect("one item in array");
    AttributeValue::insert(ctx, parrot_names_value_id, Some("samantha".into()), None)
        .await
        .expect("two items in array");
    AttributeValue::insert(ctx, parrot_names_value_id, Some("jessica".into()), None)
        .await
        .expect("three items in array");
    AttributeValue::insert(ctx, parrot_names_value_id, Some("amanda".into()), None)
        .await
        .expect("four items in array");
    AttributeValue::insert(ctx, parrot_names_value_id, Some("dr wiggles".into()), None)
        .await
        .expect("five items in array");

    // Add items to map prop
    AttributeValue::insert(
        ctx,
        treasure_map_value_id,
        Some("cheese".into()),
        Some("ohio".to_string()),
    )
    .await
    .expect("one item in map");
    AttributeValue::insert(
        ctx,
        treasure_map_value_id,
        Some("coxinha".into()),
        Some("rio".to_string()),
    )
    .await
    .expect("two items in map");
    AttributeValue::insert(
        ctx,
        treasure_map_value_id,
        Some("pizza".into()),
        Some("nyc".to_string()),
    )
    .await
    .expect("three items in map");
    AttributeValue::insert(
        ctx,
        treasure_map_value_id,
        Some("sushi".into()),
        Some("tokyo".to_string()),
    )
    .await
    .expect("four items in map");
    AttributeValue::insert(
        ctx,
        treasure_map_value_id,
        Some("baby back ribs".into()),
        Some("jupiter".to_string()),
    )
    .await
    .expect("five items in map");

    // Grab the children for the array and check that they match what they should be
    let parrot_names_child_ids =
        AttributeValue::get_child_av_ids_for_ordered_parent(ctx, parrot_names_value_id)
            .await
            .expect("get the vec of child ids");
    let parrot_names_third_item = AttributeValue::get_by_id(
        ctx,
        *parrot_names_child_ids
            .get(2)
            .expect("get the id for the third item"),
    )
    .await
    .expect("get the third item in the array");
    let parrot_names_third_item_value = parrot_names_third_item
        .value(ctx)
        .await
        .expect("get the value for this array item");

    // The value of the third item should be "jessica"
    assert_eq!(parrot_names_third_item_value, Some("jessica".into()));

    // Grab the children for the map and check that they match what they should be
    let treasure_child_ids =
        AttributeValue::get_child_av_ids_for_ordered_parent(ctx, treasure_map_value_id)
            .await
            .expect("get the vec of child ids");
    let treasure_second_item = AttributeValue::get_by_id(
        ctx,
        *treasure_child_ids
            .get(1)
            .expect("get the id for the second item"),
    )
    .await
    .expect("get the second item in the map");
    let treasure_second_item_value = treasure_second_item
        .value(ctx)
        .await
        .expect("get the value for this map item");

    // The value of the second item should be "coxinha"
    assert_eq!(treasure_second_item_value, Some("coxinha".into()));

    let treasure_second_item_key = treasure_second_item
        .key(ctx)
        .await
        .expect("get the key for this map item");

    // The key of the second item should be "rio"
    assert_eq!(treasure_second_item_key, Some("rio".to_string()));

    // Check that there are five items in the array
    assert_eq!(parrot_names_child_ids.len(), 5);

    // Check that there are five items in the map
    assert_eq!(treasure_child_ids.len(), 5);

    // ======================================================
    // Test removing items from the array
    // ======================================================

    // Remove an item from the array prop
    AttributeValue::remove_by_id(ctx, parrot_names_third_item.id())
        .await
        .expect("remove the third item in parrot_names array");
    let parrot_names_child_ids =
        AttributeValue::get_child_av_ids_for_ordered_parent(ctx, parrot_names_value_id)
            .await
            .expect("get the vec of child ids");

    // Check that there are four items in the array
    assert_eq!(parrot_names_child_ids.len(), 4);

    // Check that the items around the removed item are correct

    // Get the second item in the array
    let parrot_names_second_item = AttributeValue::get_by_id(
        ctx,
        *parrot_names_child_ids
            .get(1)
            .expect("get the second item in parrot_names"),
    )
    .await
    .expect("get the AttributeValue for the second item in parrot_names");
    let parrot_names_second_item_value = parrot_names_second_item
        .value(ctx)
        .await
        .expect("get the value for the second item in parrot_names");

    // Check that the value of the second array item is "samantha"
    assert_eq!(parrot_names_second_item_value, Some("samantha".into()));

    // Get the third item in the array
    let parrot_names_third_item = AttributeValue::get_by_id(
        ctx,
        *parrot_names_child_ids
            .get(2)
            .expect("get the third item in parrot_names"),
    )
    .await
    .expect("get the AttributeValue for the third item in parrot_names");
    let parrot_names_third_item_value = parrot_names_third_item
        .value(ctx)
        .await
        .expect("get the value for the third item in parrot_names");

    // Check that the value of the third array item is "amanda"
    assert_eq!(parrot_names_third_item_value, Some("amanda".into()));

    // ======================================================
    // Test removing items from the map
    // ======================================================

    // Remove an item from the map prop
    AttributeValue::remove_by_id(ctx, treasure_second_item.id())
        .await
        .expect("remove the second item in treasure map");
    let treasure_child_ids =
        AttributeValue::get_child_av_ids_for_ordered_parent(ctx, treasure_map_value_id)
            .await
            .expect("get the vec of child ids");

    // Check that there are four items in the array
    assert_eq!(treasure_child_ids.len(), 4);

    // Check that the items around the removed item are correct

    // Get the first item in the treasure map
    let treasure_first_item = AttributeValue::get_by_id(
        ctx,
        *treasure_child_ids.first()
            .expect("get the first item in treasure"),
    )
    .await
    .expect("get the AttributeValue for the first item in treasure");
    let treasure_first_item_value = treasure_first_item
        .value(ctx)
        .await
        .expect("get the value for the first item in treasure");

    // Check that the value of the first map item is "cheese"
    assert_eq!(treasure_first_item_value, Some("cheese".into()));

    let treasure_first_item_key = treasure_first_item
        .key(ctx)
        .await
        .expect("get the key for the first item in treasure");

    // Check that the key of the first map item is "ohio"
    assert_eq!(treasure_first_item_key, Some("ohio".to_string()));

    // Get the second item in the treasure map
    let treasure_second_item = AttributeValue::get_by_id(
        ctx,
        *treasure_child_ids
            .get(1)
            .expect("get the second item in treasure"),
    )
    .await
    .expect("get the AttributeValue for the second item in treasure");
    let treasure_second_item_value = treasure_second_item
        .value(ctx)
        .await
        .expect("get the value for the second item in treasure");

    // Check that the value of the second map item is "pizza"
    assert_eq!(treasure_second_item_value, Some("pizza".into()));

    let treasure_second_item_key = treasure_second_item
        .key(ctx)
        .await
        .expect("get the key for the second item in treasure");

    // Check that the key of the second map item is "nyc"
    assert_eq!(treasure_second_item_key, Some("nyc".to_string()));
}
