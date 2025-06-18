use dal::{
    AttributeValue,
    DalContext,
    Schema,
    SchemaVariant,
    property_editor::{
        schema::PropertyEditorSchema,
        values::PropertyEditorValues,
    },
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        PropEditorTestView,
        connect_components_with_socket_names,
        create_component_for_default_schema_name_in_default_view,
        create_component_for_schema_variant_on_default_view,
    },
    test,
};
use serde_json::json;

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
    let component = create_component_for_schema_variant_on_default_view(ctx, schema_variant_id)
        .await
        .expect("could not create component");

    // Assemble both property editor blobs.
    let _property_editor_schema = PropertyEditorSchema::assemble(ctx, schema_variant_id, false)
        .await
        .expect("could not assemble property editor schema");
    let _property_editor_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("could not assemble property editor schema");
}

#[test]
async fn array_map_manipulation(ctx: &DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "pirate", "ss poopcanoe")
            .await
            .expect("could not create component");

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
        AttributeValue::get_child_av_ids_in_order(ctx, parrot_names_value_id)
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
    let treasure_child_ids = AttributeValue::get_child_av_ids_in_order(ctx, treasure_map_value_id)
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
    AttributeValue::remove(ctx, parrot_names_third_item.id())
        .await
        .expect("remove the third item in parrot_names array");
    let parrot_names_child_ids =
        AttributeValue::get_child_av_ids_in_order(ctx, parrot_names_value_id)
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
    AttributeValue::remove(ctx, treasure_second_item.id())
        .await
        .expect("remove the second item in treasure map");
    let treasure_child_ids = AttributeValue::get_child_av_ids_in_order(ctx, treasure_map_value_id)
        .await
        .expect("get the vec of child ids");

    // Check that there are four items in the array
    assert_eq!(treasure_child_ids.len(), 4);

    // Check that the items around the removed item are correct

    // Get the first item in the treasure map
    let treasure_first_item = AttributeValue::get_by_id(
        ctx,
        *treasure_child_ids
            .first()
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

#[test]
async fn override_value_then_reset(ctx: &mut DalContext) {
    let original_pirate_name = "Thomas Cavendish";
    let pirate_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "pirate",
        original_pirate_name,
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let name_path = &["root", "domain", "name"];
    let av_id = pirate_component
        .attribute_values_for_prop(ctx, name_path)
        .await
        .expect("find value ids for the prop treasure")
        .pop()
        .expect("there should only be one value id");

    let prop_id = AttributeValue::prop_id(ctx, av_id)
        .await
        .expect("get prop_id for attribute value");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": original_pirate_name,
            "validation": null,
            "canBeSetBySocket": false,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true, // domain/name gets populated from si/name
            "overridden": false // value comes from the default prototype (schema variant context)
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(name_path)
            .expect("could not get value")
    );

    // if we set a value directly on domain/name, overridden becomes true
    let new_pirate_name = "Rock Brasiliano";
    AttributeValue::update(ctx, av_id, Some(serde_json::json!(new_pirate_name)))
        .await
        .expect("override domain/name attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": new_pirate_name,
            "validation": null,
            "canBeSetBySocket": false,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": false, // Value now comes from a si:set* function
            "overridden": true // prototype that points to function is directly for this av (component context)
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(name_path)
            .expect("could not get value")
    );

    AttributeValue::use_default_prototype(ctx, av_id)
        .await
        .expect("revert back to default prototype");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": original_pirate_name,
            "validation": null,
            "canBeSetBySocket": false,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false // value goes back to being controlled by the default function
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(name_path)
            .expect("could not get value")
    );
}

#[test]
async fn override_array_then_reset(ctx: &mut DalContext) {
    let original_pirate_name = "Thomas Cavendish";
    let pirate_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "pirate",
        original_pirate_name,
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let name_path = &["root", "domain", "parrot_names"];
    let av_id = pirate_component
        .attribute_values_for_prop(ctx, name_path)
        .await
        .expect("find value ids for the prop")
        .pop()
        .expect("there should only be one value id");

    let prop_id = AttributeValue::prop_id(ctx, av_id)
        .await
        .expect("get prop_id for attribute value");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": null,
            "validation": null,
            "canBeSetBySocket": true,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(name_path)
            .expect("could not get value")
    );

    AttributeValue::update(ctx, av_id, Some(serde_json::json!([])))
        .await
        .expect("override domain/parrot_names attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": [],
            "validation": null,
            "canBeSetBySocket": false,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": false,
            "overridden": true
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(name_path)
            .expect("could not get value")
    );

    AttributeValue::use_default_prototype(ctx, av_id)
        .await
        .expect("revert back to default prototype");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": null,
            "validation": null,
            "canBeSetBySocket": true,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false // value goes back to being controlled by the default function
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(name_path)
            .expect("could not get value")
    );
}

#[test]
async fn prop_can_be_set_by_socket(ctx: &mut DalContext) {
    let pirate_name = "Blackbeard";
    let pirate_component =
        create_component_for_default_schema_name_in_default_view(ctx, "pirate", pirate_name)
            .await
            .expect("could not create component");

    let parrots_path = &["root", "domain", "parrot_names"];

    // Check that pirate parrots can be set by socket
    let av_id = pirate_component
        .attribute_values_for_prop(ctx, parrots_path)
        .await
        .expect("find value ids for the prop treasure")
        .pop()
        .expect("there should only be one value id");

    let prop_id = AttributeValue::prop_id(ctx, av_id)
        .await
        .expect("get prop_id for attribute value");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": null,
            "validation": null,
            "canBeSetBySocket": true, // prop can be set by socket
            "isFromExternalSource": false, // prop is not getting value through that socket
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(parrots_path)
            .expect("could not get value")
    );

    let pet_shop_component =
        create_component_for_default_schema_name_in_default_view(ctx, "pet_shop", "Petopia")
            .await
            .expect("could not create component");

    connect_components_with_socket_names(
        ctx,
        pet_shop_component.id(),
        "parrot_names",
        pirate_component.id(),
        "parrot_names",
    )
    .await
    .expect("could not connect components with socket names");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(
        json![{
            "id": av_id,
            "propId": prop_id,
            "key": null,
            "value": null,
            "validation": null,
            "canBeSetBySocket": true, // prop can be set by socket
            "isFromExternalSource": true, // now that we have a connection, this is true
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(parrots_path)
            .expect("could not get value")
    );
}

#[test]
async fn values_controlled_by_ancestor(ctx: &mut DalContext) {
    let pirate_name = "Long John Silver";
    let parrot_name = "Captain Flint";
    let pirate_component =
        create_component_for_default_schema_name_in_default_view(ctx, "pirate", pirate_name)
            .await
            .expect("could not create component");

    let parrots_path = &["root", "domain", "parrot_names"];
    let parrot_entry_path = &["root", "domain", "parrot_names", "parrot_name"];

    let parrots_av_id = pirate_component
        .attribute_values_for_prop(ctx, parrots_path)
        .await
        .expect("find value ids for prop parrot_names")
        .pop()
        .expect("there should only be one value id");

    let parrots_prop_id = AttributeValue::prop_id(ctx, parrots_av_id)
        .await
        .expect("get prop_id for attribute value");

    assert_eq!(
        json![{
            "id": parrots_av_id,
            "propId": parrots_prop_id,
            "key": null,
            "value": null,
            "validation": null,
            "canBeSetBySocket": true,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(parrots_path)
            .expect("could not get value")
    );

    let pet_shop_component =
        create_component_for_default_schema_name_in_default_view(ctx, "pet_shop", "Petopia")
            .await
            .expect("could not create component");

    // set value on source component
    {
        let pet_shop_parrot_av_id = pet_shop_component
            .attribute_values_for_prop(ctx, parrots_path)
            .await
            .expect("find value ids for prop parrot_names")
            .pop()
            .expect("there should only be one value id");

        AttributeValue::insert(ctx, pet_shop_parrot_av_id, Some(parrot_name.into()), None)
            .await
            .expect("insert value in pet_shop parrot_names array");
    }

    connect_components_with_socket_names(
        ctx,
        pet_shop_component.id(),
        "parrot_names",
        pirate_component.id(),
        "parrot_names",
    )
    .await
    .expect("could not connect components with socket names");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // av for array should only change isFromExternalSource, because of the connection
    assert_eq!(
        json![{
            "id": parrots_av_id,
            "propId": parrots_prop_id,
            "key": null,
            "value": [],
            "validation": null,
            "canBeSetBySocket": true, // prop can be set by socket
            "isFromExternalSource": true, // prop gets value through that socket
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false
        }], // expected
        PropEditorTestView::for_component_id(ctx, pirate_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(parrots_path)
            .expect("could not get value")
    );

    // av for entry is controlled by ancestor
    {
        let mut parrot_entry_avs = pirate_component
            .attribute_values_for_prop(ctx, parrot_entry_path)
            .await
            .expect("find value ids for prop parrot_name");

        assert_eq!(parrot_entry_avs.len(), 1);

        let parrot_entry_av_id = parrot_entry_avs.pop().expect("there should a value id");

        let parrot_entry_prop_id = AttributeValue::prop_id(ctx, parrot_entry_av_id)
            .await
            .expect("get prop_id for attribute value");

        assert_eq!(
            json![{
                "id": parrot_entry_av_id,
                "propId": parrot_entry_prop_id,
                "key": null,
                "value": parrot_name,
                "validation": null,
                "canBeSetBySocket": false,
                "isFromExternalSource": false,
                "isControlledByAncestor": true, // this entry in the array comes from the parents function
                "isControlledByDynamicFunc": true,
                "overridden": false
            }], // expected
            PropEditorTestView::for_component_id(ctx, pirate_component.id())
                .await
                .expect("could not get property editor test view")
                .get_value(&["root", "domain", "parrot_names", "0"])
                .expect("could not get value")
        );
    }
}
