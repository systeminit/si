use dal::{
    generate_name,
    property_editor::{PropertyEditorSchema, PropertyEditorValues},
    AttributeReadContext, Component, DalContext, Schema, StandardModel,
};
use dal_test::test;

#[test]
async fn property_editor_schema(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Region".to_string())
        .await
        .expect("cannot find Region schema")
        .pop()
        .expect("no Region schema found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("missing default schema variant id");
    let _property_editor_schema = PropertyEditorSchema::for_schema_variant(ctx, *schema_variant_id)
        .await
        .expect("cannot create property editor schema from schema variant");
    // NOTE: Some day, this test should.. test something. For now, though - we'll do it live.
}

#[test]
async fn property_editor_value(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("missing default schema variant id");
    let name = generate_name();
    let (component, _node) =
        Component::new_for_schema_variant_with_node(ctx, &name, schema_variant_id)
            .await
            .expect("could not create component");

    let property_editor_values = PropertyEditorValues::for_context(
        ctx,
        AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant_id),
            component_id: Some(*component.id()),
            prop_id: None,
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("cannot create property editor values from context");

    let mut name_value = None;
    let mut image_value = None;
    for (_id, value) in property_editor_values.values {
        let prop = value
            .prop(ctx)
            .await
            .expect("could not get prop from property editor value");
        if let Some(parent_prop) = prop
            .parent_prop(ctx)
            .await
            .expect("could not perform parent prop fetch")
        {
            if prop.name() == "name" && parent_prop.name() == "si" {
                if name_value.is_some() {
                    panic!("found more than one property editor value with prop \"name\" and parent \"si\"");
                }
                name_value = Some(value.value());
            } else if prop.name() == "image" && parent_prop.name() == "domain" {
                if image_value.is_some() {
                    panic!("found more than one property editor value with prop \"image\" and parent \"domain\"");
                }
                image_value = Some(value.value());
            }
        }
    }
    let name_value = name_value
        .expect("did not find property editor value with prop \"name\" and parent \"si\"");
    let image_value = image_value
        .expect("did not find property editor value with prop \"image\" and parent \"domain\"");
    let found_name = serde_json::to_string(&name_value).expect("could not deserialize value");
    assert_eq!(found_name.replace('"', ""), name);
    assert_eq!(name_value, image_value);
}
