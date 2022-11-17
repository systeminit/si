use dal::{
    attribute::context::AttributeContextBuilder, component::view::ComponentView, generate_name,
    AttributeContext, AttributeReadContext, AttributeValue, Component, DalContext, PropKind,
    Schema, SchemaKind, StandardModel,
};
use dal_test::{
    test,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn update_for_context_simple(ctx: &DalContext) {
    // "name": String
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let name_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "name_prop").await;
    name_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of name_prop");

    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    let (component, _) = Component::new_for_schema_with_node(ctx, "Basic component", schema.id())
        .await
        .expect("Unable to create component");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "code": {},
                "domain": {},
            }
        ],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let domain_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.domain_prop_id),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get domain AttributeValue")
    .expect("domain AttributeValue not found")
    .id();
    let base_name_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*name_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get name AttributeValue")
    .expect("name AttributeValue not found");

    let update_context: AttributeContext =
        AttributeContextBuilder::from(base_attribute_read_context)
            .set_prop_id(*name_prop.id())
            .to_context()
            .expect("cannot build write AttributeContext");

    let (_, name_value_id) = AttributeValue::update_for_context(
        ctx,
        *base_name_value.id(),
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Miles".to_string()).expect("cannot create new Value")),
        None,
    )
    .await
    .expect("cannot set value for context");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "code": {},
                "domain": {
                    "name_prop": "Miles",
                },
            }
        ],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let (_, _) = AttributeValue::update_for_context(
        ctx,
        name_value_id,
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Iria".to_string()).expect("cannot create new value")),
        None,
    )
    .await
    .expect("cannot update value for context");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "code": {},
                "domain": {
                    "name_prop": "Iria",
                },
            }
        ],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );
}

#[test]
async fn insert_for_context_simple(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let array_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "array_prop").await;
    array_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of array_prop");

    let array_element = create_prop_of_kind_with_name(ctx, PropKind::String, "array_element").await;
    array_element
        .set_parent_prop(ctx, *array_prop.id())
        .await
        .expect("cannot set parent of array_element");

    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    let (component, _) = Component::new_for_schema_with_node(ctx, "Array Component", schema.id())
        .await
        .expect("Unable to create component");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "code": {},
            "domain": {},
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get array AttributeValue")
    .expect("array AttributeValue not found");
    let update_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*array_element.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let _new_array_element_value_id =
        AttributeValue::insert_for_context(ctx, update_context, *array_value.id(), None, None)
            .await
            .expect("cannot insert new array element");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "code": {},
            "domain": {
                "array_prop": [],
            },
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );
}

#[test]
async fn update_for_context_object(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let address_prop = create_prop_of_kind_with_name(ctx, PropKind::Object, "address").await;
    address_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of address_prop");

    let streets_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "streets").await;
    streets_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of streets prop");
    let streets_child_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "street").await;
    streets_child_prop
        .set_parent_prop(ctx, *streets_prop.id())
        .await
        .expect("cannot set parent of street prop");

    let city_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "city").await;
    city_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of city prop");
    let country_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "country").await;
    country_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of country prop");

    let tags_prop = create_prop_of_kind_with_name(ctx, PropKind::Map, "tags").await;
    tags_prop
        .set_parent_prop(ctx, *address_prop.id())
        .await
        .expect("cannot set parent of tags prop");
    let tags_child_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "tag").await;
    tags_child_prop
        .set_parent_prop(ctx, *tags_prop.id())
        .await
        .expect("cannot set parent of tags child prop");

    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    let (component, _) = Component::new_for_schema_with_node(ctx, "Basic component", schema.id())
        .await
        .expect("Unable to create component");

    let read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "code": {},
                "domain": {},
            }
        ],
        component_view.properties,
    );

    let root_value_id = *AttributeValue::list_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.prop_id),
            component_id: Some(*component.id()),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get root AttributeValue")
    .into_iter()
    .next()
    .expect("root AttributeValue not found")
    .id();

    let domain_value_id = *AttributeValue::list_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.domain_prop_id),
            component_id: Some(*component.id()),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get domain AttributeValue")
    .into_iter()
    .next()
    .expect("domain AttributeValue not found")
    .id();

    let update_context = AttributeContext::builder()
        .set_prop_id(root.domain_prop_id)
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let (_, domain_value_id) = AttributeValue::update_for_context(
        ctx,
        domain_value_id,
        Some(root_value_id),
        update_context,
        Some(serde_json::json!({
            "address": {
                "streets": [
                    "Suite 4",
                    "14 Main Street"
                ],
                "city": "Plainstown",
                "country": "Eurasia",
                "tags": {
                    "cool": "beans",
                    "alpha": "bet",
                },
            },
        })),
        None,
    )
    .await
    .expect("cannot update value");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "code": {},
                "domain": {
                    "address": {
                        "streets": [
                            "Suite 4",
                            "14 Main Street"
                        ],
                        "city": "Plainstown",
                        "country": "Eurasia",
                        "tags": {
                            "cool": "beans",
                            "alpha": "bet",
                        },
                    },
                },
            }
        ],
        component_view.properties,
    );

    let (_, _domain_value_id) = AttributeValue::update_for_context(
        ctx,
        domain_value_id,
        Some(root_value_id),
        update_context,
        Some(serde_json::json!({
            "address": {
                "streets": [
                    "123 Ok",
                ],
                "city": "Nowheresville",
                "tags": {
                    "new": "one",
                },
            },
        })),
        None,
    )
    .await
    .expect("cannot update value");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "code": {},
                "domain": {
                    "address": {
                        "streets": [
                            "123 Ok",
                        ],
                        "city": "Nowheresville",
                        "tags": {
                            "new": "one",
                        },
                    },
                },
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn insert_for_context_creates_array_in_final_context(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let array_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "array_prop").await;
    array_prop
        .set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("cannot set parent of array_prop");

    let array_element = create_prop_of_kind_with_name(ctx, PropKind::String, "array_element").await;
    array_element
        .set_parent_prop(ctx, *array_prop.id())
        .await
        .expect("cannot set parent of array_element");

    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    let (component, _) = Component::new_for_schema_with_node(ctx, "Array Component", schema.id())
        .await
        .expect("Unable to create component");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "code": {},
            "domain": {},
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get array AttributeValue")
    .expect("array AttributeValue not found");
    let update_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*array_element.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let _new_array_element_value_id = AttributeValue::insert_for_context(
        ctx,
        update_context,
        *array_value.id(),
        Some(serde_json::json!("Component Element")),
        None,
    )
    .await
    .expect("cannot insert new array element");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "code": {},
            "domain": {
                "array_prop": [
                    "Component Element",
                ],
            },
        }],
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties,
    );

    let _component_array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get component array AttributeValue")
    .expect("component array AttributeValue not found");
}

#[test]
async fn list_payload(ctx: &DalContext) {
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

    let payloads = AttributeValue::list_payload_for_read_context(
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
    .expect("could not list payload for read context");

    let mut name_value = None;
    let mut image_value = None;
    for payload in payloads {
        if let Some(parent_prop) = payload
            .prop
            .parent_prop(ctx)
            .await
            .expect("could not perform parent prop fetch")
        {
            if payload.prop.name() == "name" && parent_prop.name() == "si" {
                if name_value.is_some() {
                    panic!("found more than one list payload value with prop \"name\" and parent \"si\"");
                }
                name_value = Some(payload.func_binding_return_value);
            } else if payload.prop.name() == "image" && parent_prop.name() == "domain" {
                if image_value.is_some() {
                    panic!("found more than one list payload value with prop \"image\" and parent \"domain\"");
                }
                image_value = Some(payload.func_binding_return_value);
            }
        }
    }

    let name_value = name_value
        .expect("did not find list payload value with prop \"name\" and parent \"si\"")
        .expect("value is empty");
    let name_value = name_value
        .value()
        .expect("value empty for func binding return value");

    let image_value = image_value
        .expect("did not find list payload value with prop \"image\" and parent \"domain\"")
        .expect("value is empty");
    let image_value = image_value
        .value()
        .expect("value empty for func binding return value");

    let found_name = serde_json::to_string(name_value).expect("could not deserialize value");
    assert_eq!(found_name.replace('"', ""), name);
    assert_eq!(name_value, image_value);
}
