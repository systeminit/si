use pretty_assertions_sorted::assert_eq;

use dal::{
    attribute::context::AttributeContextBuilder, component::view::ComponentView, generate_name,
    AttributeContext, AttributeReadContext, AttributeValue, Component, DalContext, PropKind,
    StandardModel,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::{
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};

#[test]
async fn update_for_context_simple(ctx: &DalContext) {
    // "name": String
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let name_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "name_prop",
        PropKind::String,
        *schema_variant.id(),
        Some(root.domain_prop_id),
    )
    .await;
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

    let array_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "array_prop",
        PropKind::Array,
        *schema_variant.id(),
        Some(root.domain_prop_id),
    )
    .await;
    let array_element = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "array_element",
        PropKind::String,
        *schema_variant.id(),
        Some(*array_prop.id()),
    )
    .await;
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

    let address_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "address",
        PropKind::Object,
        schema_variant_id,
        Some(root.domain_prop_id),
    )
    .await;
    let streets_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "streets",
        PropKind::Array,
        schema_variant_id,
        Some(*address_prop.id()),
    )
    .await;
    let _streets_child_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "street",
        PropKind::String,
        schema_variant_id,
        Some(*streets_prop.id()),
    )
    .await;
    let _city_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "city",
        PropKind::String,
        schema_variant_id,
        Some(*address_prop.id()),
    )
    .await;
    let _country_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "country",
        PropKind::String,
        schema_variant_id,
        Some(*address_prop.id()),
    )
    .await;
    let tags_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "tags",
        PropKind::Map,
        schema_variant_id,
        Some(*address_prop.id()),
    )
    .await;
    let _tags_child_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "tag",
        PropKind::String,
        schema_variant_id,
        Some(*tags_prop.id()),
    )
    .await;
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

    let array_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "array_prop",
        PropKind::Array,
        schema_variant_id,
        Some(root.domain_prop_id),
    )
    .await;
    let array_element = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "array_element",
        PropKind::String,
        schema_variant_id,
        Some(*array_prop.id()),
    )
    .await;
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

#[test]
async fn use_default_prototype(ctx: &DalContext) {
    let initial_time = std::time::Instant::now();
    let mut last_checkpoint = initial_time.elapsed();

    let (identity_func_id, _, _, identity_func_argument_id) = setup_identity_func(ctx).await;

    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    // domain: Object
    // └─ object: Object
    //    ├─ source: String
    //    └─ destination: String
    let object_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "object",
        PropKind::Object,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await;
    let source_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "source",
        PropKind::String,
        schema_variant_id,
        Some(*object_prop.id()),
    )
    .await;
    let destination_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "destination",
        PropKind::String,
        schema_variant_id,
        Some(*object_prop.id()),
    )
    .await;

    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    println!("Commit 1 at {:?}", initial_time.elapsed());
    last_checkpoint = initial_time.elapsed();
    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");
    println!(
        "Executed in {:?}, {:?} since start",
        initial_time.elapsed() - last_checkpoint,
        initial_time.elapsed()
    );

    // Create connection between source and destination props
    {
        let destination_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*destination_prop.id()),
                ..AttributeReadContext::default()
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        // Find the prototype corresponding to the "destination" value (that corresponds to the
        // "destination" prop)
        let updated_destination_attribute_value =
            AttributeValue::get_by_id(ctx, destination_attribute_value.id())
                .await
                .expect("cannot find attribute value")
                .expect("attribute value not found");
        let mut destination_attribute_prototype = updated_destination_attribute_value
            .attribute_prototype(ctx)
            .await
            .expect("cannot find attribute prototype")
            .expect("attribute prototype not found");

        // Now, update the "destination" field's corresponding prototype to use the identity function
        // and the source internal provider.
        let source_internal_provider = InternalProvider::find_for_prop(ctx, *source_prop.id())
            .await
            .expect("could not get internal provider")
            .expect("internal provider not found");
        destination_attribute_prototype
            .set_func_id(ctx, identity_func_id)
            .await
            .expect("could not set func id on attribute prototype");

        // With the "source" internal provider in hand and the "destination" attribute prototype setup,
        // we can create an argument for the latter prototype.
        let _argument = AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *destination_attribute_prototype.id(),
            identity_func_argument_id,
            *source_internal_provider.id(),
        )
        .await
        .expect("could not create attribute prototype argument");

        println!("Commit 2 at {:?}", initial_time.elapsed());
        last_checkpoint = initial_time.elapsed();
        ctx.blocking_commit()
            .await
            .expect("could not commit & run jobs");
        println!(
            "Executed in {:?}, {:?} since start",
            initial_time.elapsed() - last_checkpoint,
            initial_time.elapsed()
        );
    }

    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "starfield", *schema.id())
            .await
            .expect("unable to create component");

    // Initialize the value corresponding to the "source" prop.
    {
        let object_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*object_prop.id()),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let source_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*destination_prop.id()),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let value =
            serde_json::to_value("Initial value").expect("could not convert to serde_json::Value");
        AttributeValue::update_for_context(
            ctx,
            *source_attribute_value.id(),
            Some(*object_attribute_value.id()),
            AttributeContextBuilder::from(AttributeReadContext {
                prop_id: Some(*source_prop.id()),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            })
            .to_context()
            .expect("could not convert builder to attribute context"),
            Some(value),
            None,
        )
        .await
        .expect("cannot update value for context");

        println!("Commit 3 at {:?}", initial_time.elapsed());
        last_checkpoint = initial_time.elapsed();
        ctx.blocking_commit()
            .await
            .expect("could not commit & run jobs");
        println!(
            "Executed in {:?}, {:?} since start",
            initial_time.elapsed() - last_checkpoint,
            initial_time.elapsed()
        );
    }

    // Ensure that both source and destination were updated.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "destination": "Initial value",
                    "source": "Initial value",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Override value on destination.
    let overriden_destination_av_id = {
        let destination_prop_read_ctx = AttributeReadContext {
            prop_id: Some(*destination_prop.id()),
            component_id: Some(*component.id()),
            ..AttributeReadContext::default()
        };

        let destination_write_context = AttributeContextBuilder::from(destination_prop_read_ctx)
            .to_context()
            .expect("Unable to create destination write context");
        let destination_attribute_value =
            AttributeValue::find_for_context(ctx, destination_prop_read_ctx)
                .await
                .expect("Unable to get current destination")
                .expect("AttributeValue not found");

        let parent_attribute_value_id = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(*object_prop.id()),
                component_id: Some(*component.id()),
                ..AttributeReadContext::default()
            },
        )
        .await
        .expect("Unable to get container attribute value")
        .expect("AttributeValue not found");

        let (_, overridden_attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            *destination_attribute_value.id(),
            Some(*parent_attribute_value_id.id()),
            destination_write_context,
            Some(serde_json::json!("Overridden value")),
            None,
        )
        .await
        .expect("Unable to update AttributeValue");

        println!("Commit 4 at {:?}", initial_time.elapsed());
        last_checkpoint = initial_time.elapsed();
        ctx.blocking_commit()
            .await
            .expect("could not commit & run jobs");
        println!(
            "Executed in {:?}, {:?} since start",
            initial_time.elapsed() - last_checkpoint,
            initial_time.elapsed()
        );

        overridden_attribute_value_id
    };

    // Observe that the destination field has been updated.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "destination": "Overridden value",
                    "source": "Initial value",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Reset destination to value from source
    {
        AttributeValue::use_default_prototype(ctx, overriden_destination_av_id)
            .await
            .expect("Unable to clear override");

        println!("Commit 5 at {:?}", initial_time.elapsed());
        last_checkpoint = initial_time.elapsed();
        ctx.blocking_commit()
            .await
            .expect("could not commit & run jobs");
        println!(
            "Executed in {:?}, {:?} since start",
            initial_time.elapsed() - last_checkpoint,
            initial_time.elapsed()
        );
    }

    // Observe that destination fields is back to value from source.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "destination": "Initial value",
                    "source": "Initial value",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties // actual
    );
}
