use dal::{
    attribute::context::AttributeContextBuilder, component::view::ComponentView, generate_name,
    AttributeContext, AttributeReadContext, AttributeValue, Component, DalContext, Prop, PropKind,
    StandardModel,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::{
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn update_for_context_simple(ctx: &DalContext) {
    // "name": String
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let name_prop = Prop::new(
        ctx,
        "name_prop",
        PropKind::String,
        None,
        *schema_variant.id(),
        Some(root.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "Basic component", *schema.id())
            .await
            .expect("Unable to create component");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                    "type": "component",
                    "protected": false
                },
            }
        ],
        ComponentView::new(ctx, *component.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "name_prop": "Miles",
                },
            }
        ],
        ComponentView::new(ctx, *component.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "name_prop": "Iria",
                },
            }
        ],
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties,
    );
}

#[test]
async fn insert_for_context_simple(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let array_prop = Prop::new(
        ctx,
        "array_prop",
        PropKind::Array,
        None,
        *schema_variant.id(),
        Some(root.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let array_element = Prop::new(
        ctx,
        "array_element",
        PropKind::String,
        None,
        *schema_variant.id(),
        Some(*array_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "Array Component", *schema.id())
            .await
            .expect("Unable to create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
                "type": "component",
                "protected": false
            },
        }],
        ComponentView::new(ctx, *component.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
                "type": "component",
                "protected": false
            },
            "domain": {
                "array_prop": [],
            },
        }],
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties,
    );
}

#[test]
async fn update_for_context_object(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    let address_prop = Prop::new(
        ctx,
        "address",
        PropKind::Object,
        None,
        schema_variant_id,
        Some(root.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let streets_prop = Prop::new(
        ctx,
        "streets",
        PropKind::Array,
        None,
        schema_variant_id,
        Some(*address_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let _streets_child_prop = Prop::new(
        ctx,
        "street",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*streets_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let _city_prop = Prop::new(
        ctx,
        "city",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*address_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let _country_prop = Prop::new(
        ctx,
        "country",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*address_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let tags_prop = Prop::new(
        ctx,
        "tags",
        PropKind::Map,
        None,
        schema_variant_id,
        Some(*address_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let _tags_child_prop = Prop::new(
        ctx,
        "tag",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*tags_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "Basic component", *schema.id())
            .await
            .expect("Unable to create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                    "type": "component",
                    "protected": false
                },
            }
        ],
        component_view.properties,
    );

    let root_value_id = *AttributeValue::list_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.prop_id),
            component_id: Some(*component.id()),
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "address": {
                        "city": "Plainstown",
                        "tags": {
                            "cool": "beans",
                            "alpha": "bet",
                        },
                        "country": "Eurasia",
                        "streets": [
                            "Suite 4",
                            "14 Main Street"
                        ],
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "address": {
                        "city": "Nowheresville",
                        "tags": {
                            "new": "one",
                        },
                        "streets": [
                            "123 Ok",
                        ],
                    },
                },
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn insert_for_context_creates_array_in_final_context(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    let array_prop = Prop::new(
        ctx,
        "array_prop",
        PropKind::Array,
        None,
        schema_variant_id,
        Some(root.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let array_element = Prop::new(
        ctx,
        "array_element",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*array_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "Array Component", *schema.id())
            .await
            .expect("Unable to create component");

    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
                "type": "component",
                "protected": false
            },
        }],
        ComponentView::new(ctx, *component.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
                "type": "component",
                "protected": false
            },
            "domain": {
                "array_prop": [
                    "Component Element",
                ],
            },
        }],
        ComponentView::new(ctx, *component.id())
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
    let mut bagger = ComponentBagger::new();
    let name = generate_name();
    let component_bag = bagger.create_component(ctx, &name, "starfield").await;

    ctx.blocking_commit()
        .await
        .expect("commit & wait for jobs failed");

    let payloads = AttributeValue::list_payload_for_read_context(
        ctx,
        AttributeReadContext {
            prop_id: None,
            component_id: Some(component_bag.component_id),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("could not list payload for read context");

    let mut si_name_value = None;
    let mut domain_name_value = None;
    for payload in payloads {
        if let Some(parent_prop) = payload
            .prop
            .parent_prop(ctx)
            .await
            .expect("could not perform parent prop fetch")
        {
            if payload.prop.name() == "name" && parent_prop.name() == "si" {
                if si_name_value.is_some() {
                    panic!("found more than one list payload value with prop \"name\" and parent \"si\"");
                }
                si_name_value = Some(payload.func_binding_return_value);
            } else if payload.prop.name() == "name" && parent_prop.name() == "domain" {
                if domain_name_value.is_some() {
                    panic!("found more than one list payload value with prop \"name\" and parent \"domain\"");
                }
                domain_name_value = Some(payload.func_binding_return_value);
            }
        }
    }

    let si_name_value = si_name_value
        .expect("did not find list payload value with prop \"name\" and parent \"si\"")
        .expect("value is empty");
    let si_name_value = si_name_value
        .value()
        .expect("value empty for func binding return value");

    let domain_name_value = domain_name_value
        .expect("did not find list payload value with prop \"name\" and parent \"domain\"")
        .expect("value is empty");
    let domain_name_value = domain_name_value
        .value()
        .expect("value empty for func binding return value");

    let found_name = serde_json::to_string(si_name_value).expect("could not deserialize value");
    assert_eq!(found_name.replace('"', ""), name);
    assert_eq!(si_name_value, domain_name_value);
}
