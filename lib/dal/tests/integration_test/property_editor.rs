use dal::property_editor::schema::{
    PropertyEditorProp, PropertyEditorPropKind, PropertyEditorSchema,
};
use dal::property_editor::values::{PropertyEditorValue, PropertyEditorValues};
use dal::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use dal::{AttributeValue, Component, ComponentId, DalContext, Schema, SchemaVariant};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;
use itertools::enumerate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
        *treasure_child_ids
            .get(0)
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
async fn prop_can_be_set_by_socket(ctx: &DalContext) {
    let component = create_component_for_schema_name(ctx, "starfield", "harold").await;

    let galaxies_path = &["root", "domain", "universe", "galaxies"];

    let galaxies_value_id = component
        .attribute_values_for_prop(ctx, galaxies_path)
        .await
        .expect("find value ids for the prop parrot_names")
        .pop()
        .expect("there should only be one value id");

    let galaxies_prop_id = AttributeValue::prop_id_for_id(ctx, galaxies_value_id)
        .await
        .expect("find prop id for attribute value");

    let galaxies_view = PropEditorView::for_component_id(ctx, component.id())
        .await
        .get_value(galaxies_path);

    assert_eq!(
        serde_json::json![{
            "id": galaxies_value_id,
            "propId": galaxies_prop_id,
            "key": null,
            "value": null,
            "canBeSetBySocket": true,
            "isFromExternalSource": false,
            "isControlledByAncestor": false,
            "isControlledByDynamicFunc": true,
            "overridden": false
        }], // expected
        galaxies_view, // actual
    );
}

#[derive(Serialize, Deserialize, Debug)]
struct PropEditorView {
    pub prop: PropertyEditorProp,
    pub value: PropertyEditorValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<HashMap<String, PropEditorView>>,
}
#[allow(dead_code)]
impl PropEditorView {
    fn get_view(&self, prop_path: &[&str]) -> Value {
        let mut value = serde_json::to_value(self).expect("convert UnifiedViewItem to json Value");

        // "root" is necessary for compatibility with other prop apis, but we skip it here
        for &prop_name in prop_path.iter().skip(1) {
            value = if prop_name == "*" {
                value.get("children").expect("get children entry of value")
            } else {
                value
                    .get("children")
                    .expect("get children entry of value")
                    .get(prop_name)
                    .expect("get child entry of value")
            }
            .clone();
        }

        value
    }

    fn get_prop(&self, prop_path: &[&str]) -> Value {
        let view = self.get_view(prop_path);
        view.get("prop").expect("get prop field of view").clone()
    }
    fn get_value(&self, prop_path: &[&str]) -> Value {
        let view = self.get_view(prop_path);
        view.get("value").expect("get prop field of view").clone()
    }

    async fn for_component_id(ctx: &DalContext, component_id: ComponentId) -> Self {
        let sv_id = Component::schema_variant_id(ctx, component_id)
            .await
            .expect("get schema variant from component");

        let PropertyEditorValues {
            root_value_id,
            values,
            child_values,
        } = PropertyEditorValues::assemble(ctx, component_id)
            .await
            .expect("assemble property editor values");

        let PropertyEditorSchema { props, .. } = PropertyEditorSchema::assemble(ctx, sv_id)
            .await
            .expect("assemble property editor schema");

        let root_view = {
            let value = values
                .get(&root_value_id)
                .expect("get property editor root value")
                .clone();

            let prop = props.get(&value.prop_id).expect("get property editor prop");

            PropEditorView {
                prop: prop.clone(),
                value,
                children: Self::property_editor_compile_children(
                    root_value_id,
                    &prop.kind,
                    &values,
                    &child_values,
                    &props,
                ),
            }
        };

        root_view
    }

    fn property_editor_compile_children(
        parent_value_id: PropertyEditorValueId,
        parent_prop_kind: &PropertyEditorPropKind,
        values: &HashMap<PropertyEditorValueId, PropertyEditorValue>,
        child_values: &HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
        props: &HashMap<PropertyEditorPropId, PropertyEditorProp>,
    ) -> Option<HashMap<String, PropEditorView>> {
        let mut children = HashMap::new();

        for (index, child_id) in enumerate(
            child_values
                .get(&parent_value_id)
                .expect("get prop editor value children"),
        ) {
            let value = values
                .get(child_id)
                .expect("get property editor root value")
                .clone();

            let prop = props.get(&value.prop_id).expect("get property editor prop");

            let key = match parent_prop_kind {
                PropertyEditorPropKind::Array => index.to_string(),
                PropertyEditorPropKind::Map => value.key.clone().unwrap_or("ERROR".to_string()),
                _ => prop.name.clone(),
            };

            let child = PropEditorView {
                prop: prop.clone(),
                value,
                children: Self::property_editor_compile_children(
                    *child_id,
                    &prop.kind,
                    values,
                    child_values,
                    props,
                ),
            };

            children.insert(key, child);
        }

        if children.is_empty() {
            None
        } else {
            Some(children)
        }
    }
}
