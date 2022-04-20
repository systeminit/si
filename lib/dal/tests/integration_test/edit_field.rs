use crate::dal::test;
use dal::attribute::context::AttributeContextBuilder;
use dal::edit_field::{EditFieldBaggage, Widget};
use dal::DalContext;
use dal::{
    component::Component,
    edit_field::{EditField, EditFieldAble, EditFieldDataType},
    test_harness::{
        create_prop_of_kind_and_set_parent_with_name, create_schema,
        create_schema_variant_with_root,
    },
    AttributeReadContext, PropKind, SchemaKind, StandardModel,
};
use pretty_assertions_sorted::assert_eq;

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
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // domain: Object
    // └─ object: Object
    //    ├─ name: String
    //    └─ value: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root.domain_prop_id,
        base_attribute_read_context,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;
    let _value_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "value",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;

    let (component, _, _) = Component::new_for_schema_with_node(ctx, "radahn", schema.id())
        .await
        .expect("cannot create component");

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
    ];

    found.sort_by_key(|v| v.0);
    expected.sort_by_key(|v| v.0);
    assert_eq!(found, expected);
}

#[test]
async fn update_edit_field_for_component(ctx: &DalContext<'_, '_>) {
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
    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // domain: Object
    // └─ object: Object
    //    ├─ name: String
    //    └─ value: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root.domain_prop_id,
        base_attribute_read_context,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;
    let _value_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "value",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;

    let (component, _, _) = Component::new_for_schema_with_node(ctx, "radahn", schema.id())
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

    let old_baggage = old_edit_field
        .baggage()
        .clone()
        .expect("baggage not found on edit field");

    let attribute_context = AttributeContextBuilder::new()
        .set_prop_id(old_baggage.prop_id)
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("could not create attribute context from builder");

    let new_value = Some(serde_json::json!("Aubrey Drake Graham"));
    Component::update_from_edit_field_with_baggage(
        ctx,
        new_value.clone(),
        attribute_context,
        old_baggage,
    )
    .await
    .expect("could not update from edit field");

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

fn select_edit_field_by_id(edit_fields: &Vec<EditField>, id: &str) -> Option<EditField> {
    for edit_field in edit_fields {
        if edit_field.id() == id {
            return Some(edit_field.to_owned());
        }
    }
    return None;
}

fn recursive_edit_fields(edit_fields: Vec<EditField>) -> Vec<EditField> {
    let mut temp: Vec<EditField> = Vec::new();
    for edit_field in &edit_fields {
        // FIXME(nick): this way of getting children only works for Object! It does not work for
        // Arrays and Maps... unless they end up using Widget::Header.
        if let Widget::Header(header) = edit_field.widget() {
            temp.append(&mut recursive_edit_fields(header.edit_fields().to_vec()));
        }
    }
    let mut combined = edit_fields;
    combined.append(&mut temp);
    combined
}
