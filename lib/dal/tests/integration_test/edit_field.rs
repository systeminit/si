use crate::dal::test;
use dal::attribute::context::read::UNSET_ID_VALUE;
use dal::attribute::context::AttributeContextBuilder;
use dal::edit_field::{EditFieldBaggage, Widget};
use dal::{
    component::Component,
    edit_field::{EditField, EditFieldAble, EditFieldDataType},
    test_harness::{
        create_prop_of_kind_and_set_parent_with_name, create_schema,
        create_schema_variant_with_root,
    },
    AttributeContext, AttributeReadContext, AttributeValueId, ComponentId, ComponentView, PropId,
    PropKind, SchemaId, SchemaKind, SchemaVariantId, StandardModel,
};
use dal::{AttributeValue, DalContext};
use pretty_assertions_sorted::assert_eq;
use std::collections::HashMap;

#[test]
async fn get_edit_fields_for_component(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema_variant
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // Create all props we want to have edit fields for.
    //
    // domain: Object
    // └─ object: Object
    //    ├─ name: String
    //    ├─ value: String
    //    └─ array_of_maps: Array
    //       └─ map_of_arrays: Map
    //          └─ array_of_integers: Array of (elements: Integer)
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root.domain_prop_id,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
    )
    .await;
    let _value_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "value",
        *object_prop.id(),
    )
    .await;
    let array_of_maps_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Array,
        "array_of_maps",
        *object_prop.id(),
    )
    .await;
    let map_of_arrays_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Map,
        "map_of_arrays",
        *array_of_maps_prop.id(),
    )
    .await;
    let array_of_integers_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Array,
        "array_of_integers",
        *map_of_arrays_prop.id(),
    )
    .await;
    let _elements_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Integer,
        "elements",
        *array_of_integers_prop.id(),
    )
    .await;

    let (component, _) = Component::new_for_schema_with_node(ctx, "radahn", schema.id())
        .await
        .expect("cannot create component");

    let base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    let edit_fields = recursive_edit_fields(base_edit_fields);
    panic!();

    // Initialize values for the lineage from the integer array elements up.
    //
    // domain: Object (already set)
    // └─ object: Object (need to set)
    //    └─ array_of_maps: Array (need to set)
    //       └─ map_of_arrays: Map (need to set)
    //          └─ array_of_integers: Array (need to set)
    update_attribute_value(
        ctx,
        Some(serde_json::json![{}]),
        object_prop.id(),
        schema.id(),
        schema_variant.id(),
        component.id(),
    )
    .await;
    update_attribute_value(
        ctx,
        Some(serde_json::json![[]]),
        array_of_maps_prop.id(),
        schema.id(),
        schema_variant.id(),
        component.id(),
    )
    .await;
    update_attribute_value(
        ctx,
        Some(serde_json::json![{}]),
        map_of_arrays_prop.id(),
        schema.id(),
        schema_variant.id(),
        component.id(),
    )
    .await;
    update_attribute_value(
        ctx,
        Some(serde_json::json![[]]),
        array_of_integers_prop.id(),
        schema.id(),
        schema_variant.id(),
        component.id(),
    )
    .await;

    let base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    let edit_fields = recursive_edit_fields(base_edit_fields);

    let mut found: Vec<(&str, &EditFieldDataType)> = edit_fields
        .iter()
        .map(|v| (v.id(), v.data_type()))
        .collect();

    let mut expected = vec![
        ("properties.root", &EditFieldDataType::Object),
        ("properties.root.si", &EditFieldDataType::Object),
        ("properties.root.domain", &EditFieldDataType::Object),
        ("properties.root.si.name", &EditFieldDataType::String),
        ("properties.root.domain.object", &EditFieldDataType::Object),
        (
            "properties.root.domain.object.name",
            &EditFieldDataType::String,
        ),
        (
            "properties.root.domain.object.value",
            &EditFieldDataType::String,
        ),
        (
            "properties.root.domain.object.array_of_maps",
            &EditFieldDataType::Array,
        ),
        (
            "properties.root.domain.object.array_of_maps.map_of_arrays",
            &EditFieldDataType::Map,
        ),
        (
            "properties.root.domain.object.array_of_maps.map_of_arrays.array_of_integers",
            &EditFieldDataType::Array,
        ),
        (
            "properties.root.domain.object.array_of_maps.map_of_arrays.array_of_integers.elements",
            &EditFieldDataType::Integer,
        ),
    ];

    found.sort_by_key(|v| v.0);
    expected.sort_by_key(|v| v.0);
    assert_eq!(found, expected);

    let view = ComponentView::for_context(
        ctx,
        AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*component.id()),
            system_id: None,
        },
    )
    .await
    .expect("could not create component view");

    // SHOULD FAIL
    assert_eq!(view.properties, serde_json::json!("{}"));
}

#[test]
async fn update_edit_field_for_component_string(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema_variant
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // domain: Object
    // └─ object: Object
    //    ├─ name: String
    //    └─ value: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root.domain_prop_id,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
    )
    .await;
    let _value_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "value",
        *object_prop.id(),
    )
    .await;

    let (component, _) = Component::new_for_schema_with_node(ctx, "radahn", schema.id())
        .await
        .expect("cannot create component");

    let old_base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    let old_edit_fields = recursive_edit_fields(old_base_edit_fields);

    let target_edit_field_id = "properties.root.domain.object.value";
    let old_edit_field = select_edit_field_by_id(&old_edit_fields, target_edit_field_id)
        .expect("could not find edit field by id");
    assert_eq!(*old_edit_field.value(), None);

    let new_value = Some(serde_json::json!("Aubrey Drake Graham"));
    update_edit_field_for_component(
        ctx,
        &old_edit_field,
        new_value.clone(),
        *schema.id(),
        *schema_variant.id(),
        *component.id(),
    )
    .await;

    // Check if the value was updated properly.
    let updated_base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    let updated_edit_fields = recursive_edit_fields(updated_base_edit_fields);
    let updated_edit_field = select_edit_field_by_id(&updated_edit_fields, target_edit_field_id)
        .expect("could not find edit field by id");
    assert_eq!(*updated_edit_field.value(), new_value);

    // Ensure that no other values were changed in the process of updating a sole edit field.
    let mut old_values: Vec<(&str, &Option<serde_json::Value>, &Option<EditFieldBaggage>)> =
        old_edit_fields
            .iter()
            .map(|v| (v.id(), v.value(), v.baggage()))
            .collect();
    old_values.retain(|v| v.0 != target_edit_field_id);
    old_values.sort_by_key(|v| v.0);

    let mut updated_values: Vec<(&str, &Option<serde_json::Value>, &Option<EditFieldBaggage>)> =
        updated_edit_fields
            .iter()
            .map(|v| (v.id(), v.value(), v.baggage()))
            .collect();
    updated_values.retain(|v| v.0 != target_edit_field_id);
    updated_values.sort_by_key(|v| v.0);

    assert_eq!(old_values, updated_values);
}

#[test]
async fn update_edit_field_for_component_array(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema_variant
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // domain: Object
    // └─ object: Object
    //    ├─ name: String
    //    └─ array: Array of (elements: Integer)
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root.domain_prop_id,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
    )
    .await;
    let array_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Array,
        "array",
        *object_prop.id(),
    )
    .await;
    let array_elements_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Integer,
        "elements",
        *array_prop.id(),
    )
    .await;

    let (component, _) = Component::new_for_schema_with_node(ctx, "tinytina", schema.id())
        .await
        .expect("cannot create component");

    let base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    let edit_fields = recursive_edit_fields(base_edit_fields);

    // Ensure the direct lineage to the array elements is initialized. Let's collect the lineage
    // first.
    let mut object_edit_field = None;
    let mut array_edit_field = None;
    for edit_field in &edit_fields {
        if edit_field.id() == "properties.root.domain.object" {
            object_edit_field = Some(edit_field.clone());
        } else if edit_field.id() == "properties.root.domain.object.array" {
            array_edit_field = Some(edit_field.clone());
        }
    }

    update_edit_field_for_component(
        ctx,
        &object_edit_field.expect("could not find object edit field"),
        Some(serde_json::json!("{}")),
        *schema.id(),
        *schema_variant.id(),
        *component.id(),
    )
    .await;

    let initialized_base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    assert_eq!(initialized_base_edit_fields, vec![]);
    let foo = recursive_edit_fields(initialized_base_edit_fields);
    panic!("EJECT");

    let domain_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        system_id: None,
    };

    let view = ComponentView::for_context(ctx, domain_attribute_read_context)
        .await
        .expect("could not create component view");

    assert_eq!(view.properties, serde_json::json!("{}"));

    let (array_baggage, _array_attribute_context) = update_edit_field_for_component(
        ctx,
        &array_edit_field.expect("could not find array edit field"),
        Some(serde_json::json!("[]")),
        *schema.id(),
        *schema_variant.id(),
        *component.id(),
    )
    .await;

    let domain_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        system_id: None,
    };

    let view = ComponentView::for_context(ctx, domain_attribute_read_context)
        .await
        .expect("could not create component view");

    assert_eq!(view.properties, serde_json::json!("{}"));

    // // Now, let's insert two values into our array.
    // let array_elements_attribute_context = AttributeContextBuilder::new()
    //     .set_prop_id(*array_elements_prop.id())
    //     .set_schema_id(*schema.id())
    //     .set_schema_variant_id(*schema_variant.id())
    //     .set_component_id(*component.id())
    //     .to_context()
    //     .expect("could not get attribute context from builder");
    // let _first_inserted_value_id = AttributeValue::insert_for_context(
    //     ctx,
    //     array_elements_attribute_context,
    //     array_baggage.attribute_value_id,
    //     Some(serde_json::json!(serde_json::Number::from(1))),
    //     array_baggage.key.clone(),
    // )
    // .await
    // .expect("could not insert for context");
    // let second_inserted_value_id = AttributeValue::insert_for_context(
    //     ctx,
    //     array_elements_attribute_context,
    //     array_baggage.attribute_value_id,
    //     Some(serde_json::json!(serde_json::Number::from(2))),
    //     array_baggage.key.clone(),
    // )
    // .await
    // .expect("could not insert for context");

    // Let's update the second value that we inserted. We'll need to get the edit fields again.
    let initialized_base_edit_fields = Component::get_edit_fields(ctx, component.id())
        .await
        .expect("could not get edit fields");
    let initialized_edit_fields = recursive_edit_fields(initialized_base_edit_fields);

    ///////////////////////////////////////////////////////////////////////////////////

    let domain_attribute_read_context = AttributeReadContext {
        prop_id: Some(UNSET_ID_VALUE.into()),
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        system_id: Some(UNSET_ID_VALUE.into()),
    };

    let view = ComponentView::for_context(ctx, domain_attribute_read_context)
        .await
        .expect("could not create component view");

    assert_eq!(view.properties, serde_json::json!("{}"));

    let mut foo: Vec<(String, Option<AttributeValueId>)> = Vec::new();
    for v in initialized_edit_fields.clone() {
        let val = v.baggage().clone().unwrap().attribute_value_id;
        foo.push((v.id().to_string(), Some(val)));
    }
    assert_eq!(foo, vec![(format!("{:?}", root.domain_prop_id), None)]);

    ///////////////////////////////////////////////////////////////////////////////////

    // let second_element_edit_field =
    //     select_edit_field_by_attribute_value_id(&initialized_edit_fields, second_inserted_value_id)
    //         .expect("could not find edit field by attribute value id");
    // update_edit_field_for_component(
    //     ctx,
    //     &second_element_edit_field,
    //     Some(serde_json::json!(serde_json::Number::from(9000))),
    //     *schema.id(),
    //     *schema_variant.id(),
    //     *component.id(),
    // )
    // .await;

    let domain_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        system_id: None,
    };

    let view = ComponentView::for_context(ctx, domain_attribute_read_context)
        .await
        .expect("could not create component view");

    assert_eq!(view.properties, serde_json::json!("{}"));
}

async fn update_attribute_value(
    ctx: &DalContext<'_, '_>,
    value: Option<serde_json::Value>,
    prop_id: &PropId,
    schema_id: &SchemaId,
    schema_variant_id: &SchemaVariantId,
    component_id: &ComponentId,
) {
    let attribute_value = AttributeValue::find_for_prop(ctx, *prop_id)
        .await
        .expect("could not find attribute value for prop");
    let parent_attribute_value = attribute_value
        .parent_attribute_value(ctx)
        .await
        .expect("could not get parent for attribute value")
        .expect("parent not found for attribute value");
    let context = AttributeContextBuilder::new()
        .set_prop_id(*prop_id)
        .set_schema_id(*schema_id)
        .set_schema_variant_id(*schema_variant_id)
        .set_component_id(*component_id)
        .to_context()
        .expect("could not convert builder to attribute context");
    AttributeValue::update_for_context(
        ctx,
        *attribute_value.id(),
        Some(*parent_attribute_value.id()),
        context,
        value,
        attribute_value.key,
    )
    .await
    .expect("could not update attribute value for context");
}

fn select_edit_field_by_id(edit_fields: &Vec<EditField>, id: &str) -> Option<EditField> {
    for edit_field in edit_fields {
        if edit_field.id() == id {
            return Some(edit_field.to_owned());
        }
    }
    return None;
}

fn select_edit_field_by_attribute_value_id(
    edit_fields: &Vec<EditField>,
    attribute_value_id: AttributeValueId,
) -> Option<EditField> {
    for edit_field in edit_fields {
        let baggage = edit_field
            .baggage()
            .as_ref()
            .expect("no baggage on edit field");
        if baggage.attribute_value_id == attribute_value_id {
            return Some(edit_field.to_owned());
        }
    }
    return None;
}

/// This function compiles all edit fields found on a nested root edit field by using its widget
/// field. Technically, the resulting list contains duplicate data since every edit field contains
/// its children within its widget field (if applicable).
// NOTE(nick): what this function's calling tests try to do can likely be replaced with using
// ComponentView for comparison (e.g. did other shit break?). Thus, this test can likely be
// removed as well if that happens.
fn recursive_edit_fields(edit_fields: Vec<EditField>) -> Vec<EditField> {
    let mut temp: Vec<EditField> = Vec::new();
    for edit_field in &edit_fields {
        if let Widget::Header(header) = edit_field.widget() {
            temp.append(&mut recursive_edit_fields(header.edit_fields().to_vec()));
        } else if let Widget::Array(array) = edit_field.widget() {
            temp.append(&mut recursive_edit_fields(array.entries().to_vec()));
        }
    }
    let mut combined = edit_fields;
    combined.append(&mut temp);
    combined
}

// FIXME(nick): the incessant baggage cloning must cease!
async fn update_edit_field_for_component(
    ctx: &DalContext<'_, '_>,
    edit_field: &EditField,
    value: Option<serde_json::Value>,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    component_id: ComponentId,
) -> (EditFieldBaggage, AttributeContext) {
    let baggage = edit_field
        .baggage()
        .clone()
        .expect("baggage not found on edit field");

    let attribute_context = AttributeContextBuilder::new()
        .set_prop_id(baggage.prop_id)
        .set_schema_id(schema_id)
        .set_schema_variant_id(schema_variant_id)
        .set_component_id(component_id)
        .to_context()
        .expect("could not create attribute context from builder");

    Component::update_from_edit_field_with_baggage(&ctx, value, attribute_context, baggage.clone())
        .await
        .expect("could not update edit field for component");

    (baggage, attribute_context)
}
