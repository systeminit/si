use dal::{
    socket::SocketArity, AttributeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, Component, ComponentView, DalContext, Edge, ExternalProvider,
    ExternalProviderId, InternalProvider, InternalProviderId, Prop, PropId, PropKind,
    StandardModel,
};
use dal_test::{
    helpers::{component_bag::ComponentBag, setup_identity_func},
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn inter_component_identity_update(ctx: &DalContext) {
    let (
        identity_func_id,
        _identity_func_binding_id,
        _identity_func_binding_return_value_id,
        id_func_arg_id,
    ) = setup_identity_func(ctx).await;

    // Setup both components used for inter component identity update.
    let (esp_bag, source_prop_id, intermediate_prop_id, esp_external_provider_id) =
        setup_esp(ctx).await;
    let (swings_bag, _destination_prop_id, swings_explicit_internal_provider_id) =
        setup_swings(ctx).await;

    // Ensure setup went as expected.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "esp",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "source": "zero-source",
                    "intermediate": "zero-intermediate",
                },
            },
        }], // expected
        esp_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "swings",
                "type": "component",
                "protected": false
            },
        }], // expected
        swings_bag.component_view_properties_raw(ctx).await // actual
    );
    // Setup the "esp" intra component update functionality from "source" to "intermediate".
    let intermediate_attribute_value = AttributeValue::find_for_context(
        ctx,
        esp_bag.attribute_read_context_with_prop(intermediate_prop_id),
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let mut intermediate_attribute_prototype = intermediate_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("cannot find attribute prototype")
        .expect("attribute prototype not found");
    intermediate_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await
        .expect("could not set func id on attribute prototype");
    let source_internal_provider = InternalProvider::find_for_prop(ctx, source_prop_id)
        .await
        .expect("could not get internal provider")
        .expect("internal provider not found");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *intermediate_attribute_prototype.id(),
        id_func_arg_id,
        *source_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Update the "esp" field, "source", to see if the intra component connection continues to work.
    esp_bag
        .update_attribute_value_for_prop(ctx, source_prop_id, Some(serde_json::json!["one"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that they look as we expect.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "esp",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "source": "one",
                    "intermediate": "one",
                },
            },
        }], // expected
        esp_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "swings",
                "type": "component",
                "protected": false
            },
        }], // expected
        swings_bag.component_view_properties_raw(ctx).await // actual
    );

    // Ensure that both components look as we expect when not "connected". The creation of both the
    // "esp" external provider and the "swings" implicit internal provider should not affect intra
    // component identity update working.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "esp",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "intermediate": "one",
                    "source": "one",
                },
            },
        }], // expected
        esp_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "swings",
                "type": "component",
                "protected": false
            },
        }], // expected
        swings_bag.component_view_properties_raw(ctx).await // actual
    );

    // Connect the two components.
    Edge::connect_providers_for_components(
        ctx,
        swings_explicit_internal_provider_id,
        swings_bag.component_id,
        esp_external_provider_id,
        esp_bag.component_id,
    )
    .await
    .expect("could not connect providers");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that both components continue to look as we expect.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "esp",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "intermediate": "one",
                    "source": "one",
                },
            },
        }], // expected
        esp_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "swings",
                "type": "component",
                "protected": false
            },
        }], // expected
        swings_bag.component_view_properties_raw(ctx).await // actual
    );

    // Update the "esp" field, "source", again.
    esp_bag
        .update_attribute_value_for_prop(ctx, source_prop_id, Some(serde_json::json!["two"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that inter component identity updating work.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "esp",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "intermediate": "two",
                    "source": "two",
                },
            },
        }], // expected
        esp_bag.component_view_properties_raw(ctx).await // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "swings",
                "type": "component",
                "protected": false
            },
            "domain": {
                "destination": "two",
            },
        }], // expected
        swings_bag.component_view_properties_raw(ctx).await // actual
    );
}

// 38.805354552534816, -77.05091482877533
async fn setup_esp(ctx: &DalContext) -> (ComponentBag, PropId, PropId, ExternalProviderId) {
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        id_func_arg_id,
    ) = setup_identity_func(ctx).await;

    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    // "esp"
    // domain: Object
    // └─ object: Object
    //    ├─ source: String
    //    └─ intermediate: String
    let object_prop = Prop::new(
        ctx,
        "object",
        PropKind::Object,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let source_prop = Prop::new(
        ctx,
        "source",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let intermediate_prop = Prop::new(
        ctx,
        "intermediate",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");

    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    // Create the "esp" external provider for inter component connection.
    let (esp_external_provider, _socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "output",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        "output",
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create external provider");

    let esp_intermediate_internal_provider =
        InternalProvider::find_for_prop(ctx, *intermediate_prop.id())
            .await
            .expect("could not get internal provider")
            .expect("internal provider not found");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *esp_external_provider
            .attribute_prototype_id()
            .expect("no attribute prototype id for external provider"),
        id_func_arg_id,
        *esp_intermediate_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, node) =
        Component::new_for_default_variant_from_schema(ctx, "esp", *schema.id())
            .await
            .expect("unable to create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // The base attribute read context can also be used for generating component views.
    let component_bag = ComponentBag {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        component_id: *component.id(),
        node_id: *node.id(),
        base_attribute_read_context: AttributeReadContext {
            prop_id: None,
            component_id: Some(*component.id()),
            ..AttributeReadContext::default()
        },
    };

    // Initialize the value corresponding to the "source" prop.
    component_bag
        .update_attribute_value_for_prop(
            ctx,
            *source_prop.id(),
            Some(serde_json::json!["zero-source"]),
        )
        .await;

    // Initialize the value corresponding to the "intermediate" prop.
    component_bag
        .update_attribute_value_for_prop(
            ctx,
            *intermediate_prop.id(),
            Some(serde_json::json!["zero-intermediate"]),
        )
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Return the bag and prop(s) used for future updates.
    (
        component_bag,
        *source_prop.id(),
        *intermediate_prop.id(),
        *esp_external_provider.id(),
    )
}

// 38.82091849697006, -77.05236860190759
async fn setup_swings(ctx: &DalContext) -> (ComponentBag, PropId, InternalProviderId) {
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        id_func_arg_id,
    ) = setup_identity_func(ctx).await;

    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    // "swings"
    // domain: Object
    // └─ destination: string
    let destination_prop = Prop::new(
        ctx,
        "destination",
        PropKind::String,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");

    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    // Create the "swings" explicit internal provider for intra component connection.
    let (swings_explicit_internal_provider, _socket) = InternalProvider::new_explicit_with_socket(
        ctx,
        *schema_variant.id(),
        "swings",
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        "swings",
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create explicit internal provider");

    let swings_destination_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*destination_prop.id()),
            ..Default::default()
        },
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");

    let mut swings_destination_attribute_prototype = swings_destination_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not find attribute prototype")
        .expect("attribute prototype not found");
    swings_destination_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await
        .expect("could not set func id on attribute prototype");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *swings_destination_attribute_prototype.id(),
        id_func_arg_id,
        *swings_explicit_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, node) =
        Component::new_for_default_variant_from_schema(ctx, "swings", *schema.id())
            .await
            .expect("unable to create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // This context can also be used for generating component views.
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    // Return the bag and prop(s) used for future updates.
    (
        ComponentBag {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            component_id: *component.id(),
            node_id: *node.id(),
            base_attribute_read_context,
        },
        *destination_prop.id(),
        *swings_explicit_internal_provider.id(),
    )
}

#[test]
async fn with_deep_data_structure(ctx: &DalContext) {
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        id_func_arg_id,
    ) = setup_identity_func(ctx).await;

    let mut source_schema = create_schema(ctx).await;
    let (mut source_schema_variant, source_root) =
        create_schema_variant_with_root(ctx, *source_schema.id()).await;
    source_schema
        .set_default_schema_variant_id(ctx, Some(*source_schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let source_object_prop = Prop::new(
        ctx,
        "base_object",
        PropKind::Object,
        None,
        *source_schema_variant.id(),
        Some(source_root.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let source_foo_prop = Prop::new(
        ctx,
        "foo_string",
        PropKind::String,
        None,
        *source_schema_variant.id(),
        Some(*source_object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let source_bar_prop = Prop::new(
        ctx,
        "bar_string",
        PropKind::String,
        None,
        *source_schema_variant.id(),
        Some(*source_object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    source_schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize source SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (source_external_provider, _socket) = ExternalProvider::new_with_socket(
        ctx,
        *source_schema.id(),
        *source_schema_variant.id(),
        "source_data",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        "source_data",
        SocketArity::Many,
        false,
    )
    .await
    .expect("cannot create source external provider");
    let source_internal_provider = InternalProvider::find_for_prop(ctx, *source_object_prop.id())
        .await
        .expect("cannot get source internal provider")
        .expect("source internal provider not found");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *source_external_provider
            .attribute_prototype_id()
            .expect("no attribute prototype id for external provider"),
        id_func_arg_id,
        *source_internal_provider.id(),
    )
    .await
    .expect("cannot create source external provider attribute prototype argument");

    let mut destination_schema = create_schema(ctx).await;
    let (mut destination_schema_variant, destination_root) =
        create_schema_variant_with_root(ctx, *destination_schema.id()).await;
    destination_schema
        .set_default_schema_variant_id(ctx, Some(*destination_schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let destination_parent_object_prop = Prop::new(
        ctx,
        "parent_object",
        PropKind::Object,
        None,
        *destination_schema_variant.id(),
        Some(destination_root.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let destination_object_prop = Prop::new(
        ctx,
        "base_object",
        PropKind::Object,
        None,
        *destination_schema_variant.id(),
        Some(*destination_parent_object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let destination_foo_prop = Prop::new(
        ctx,
        "foo_string",
        PropKind::String,
        None,
        *destination_schema_variant.id(),
        Some(*destination_object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let _destination_bar_prop = Prop::new(
        ctx,
        "bar_string",
        PropKind::String,
        None,
        *destination_schema_variant.id(),
        Some(*destination_object_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    destination_schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize destination SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let destination_object_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*destination_object_prop.id()),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("cannot find destination attribute value")
    .expect("destination attribute value not found");
    let mut destination_object_prototype = destination_object_value
        .attribute_prototype(ctx)
        .await
        .expect("cannot find attribute prototype")
        .expect("attribute prototype not found");
    destination_object_prototype
        .set_func_id(ctx, identity_func_id)
        .await
        .expect("cannot set function on destination object prototype");
    let (destination_internal_provider, _socket) = InternalProvider::new_explicit_with_socket(
        ctx,
        *destination_schema_variant.id(),
        "destination_data",
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        "destination_data",
        SocketArity::One,
        false,
    )
    .await
    .expect("cannot create destination explicit internal provider");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *destination_object_prototype.id(),
        id_func_arg_id,
        *destination_internal_provider.id(),
    )
    .await
    .expect("cannot create prototype argument for destination");

    let (source_component, _) = Component::new_for_default_variant_from_schema(
        ctx,
        "Source Component",
        *source_schema.id(),
    )
    .await
    .expect("Unable to create source component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let source_attribute_read_context = AttributeReadContext {
        prop_id: None,
        component_id: Some(*source_component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Source Component",
                    "type": "component",
                    "protected": false
                },
            }
        ],
        ComponentView::new(ctx, *source_component.id())
            .await
            .expect("cannot get source component view")
            .properties,
    );

    let (destination_component, _) = Component::new_for_default_variant_from_schema(
        ctx,
        "Destination Component",
        *destination_schema.id(),
    )
    .await
    .expect("Unable to create destination component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Destination Component",
                    "type": "component",
                    "protected": false
                },
            }
        ],
        ComponentView::new(ctx, *destination_component.id())
            .await
            .expect("cannot get destination component view")
            .properties,
    );

    Edge::connect_providers_for_components(
        ctx,
        *destination_internal_provider.id(),
        *destination_component.id(),
        *source_external_provider.id(),
        *source_component.id(),
    )
    .await
    .expect("could not connect providers");

    let _source_domain_attribute_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(source_root.domain_prop_id),
            ..source_attribute_read_context
        },
    )
    .await
    .expect("cannot get source domain AttributeValue")
    .expect("source domain AttributeValue not found")
    .id();

    let source_object_attribute_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*source_object_prop.id()),
            ..source_attribute_read_context
        },
    )
    .await
    .expect("cannot get source object AttributeValue")
    .expect("source object AttributeValue not found")
    .id();

    let source_foo_attribute_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*source_foo_prop.id()),
            ..source_attribute_read_context
        },
    )
    .await
    .expect("cannot get source foo AttributeValue")
    .expect("source foo AttributeValue not found")
    .id();

    let source_foo_update_context = AttributeContext::builder()
        .set_prop_id(*source_foo_prop.id())
        .set_component_id(*source_component.id())
        .to_context()
        .expect("could not create source foo update context");

    let (_, _) = AttributeValue::update_for_context(
        ctx,
        source_foo_attribute_value_id,
        Some(source_object_attribute_value_id),
        source_foo_update_context,
        Some(serde_json::to_value("deep update").expect("could not convert to serde_json::Value")),
        None,
    )
    .await
    .expect("cannot update source foo_string");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Source Component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "base_object": {
                        "foo_string": "deep update",
                    },
                },
            }
        ],
        ComponentView::new(ctx, *source_component.id())
            .await
            .expect("cannot get source component view")
            .properties,
    );

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Destination Component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "parent_object": {
                        "base_object": {
                            "foo_string": "deep update",
                        },
                    },
                },
            }
        ],
        ComponentView::new(ctx, *destination_component.id())
            .await
            .expect("cannot get destination component view")
            .properties,
    );

    let source_object_attribute_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*source_object_prop.id()),
            ..source_attribute_read_context
        },
    )
    .await
    .expect("cannot get source object AttributeValue")
    .expect("source object AttributeValue not found")
    .id();
    let source_bar_attribute_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*source_bar_prop.id()),
            ..source_attribute_read_context
        },
    )
    .await
    .expect("cannot get source bar AttributeValue")
    .expect("source foo AttributeValue not found")
    .id();

    let source_bar_update_context = AttributeContext::builder()
        .set_prop_id(*source_bar_prop.id())
        .set_component_id(*source_component.id())
        .to_context()
        .expect("could not create source foo update context");

    let (_, _) = AttributeValue::update_for_context(
        ctx,
        source_bar_attribute_value_id,
        Some(source_object_attribute_value_id),
        source_bar_update_context,
        Some(
            serde_json::to_value("another update").expect("could not convert to serde_json::Value"),
        ),
        None,
    )
    .await
    .expect("cannot update source bar_string");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Source Component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "base_object": {
                        "foo_string": "deep update",
                        "bar_string": "another update",
                    },
                },
            }
        ],
        ComponentView::new(ctx, *source_component.id())
            .await
            .expect("cannot get source component view")
            .properties,
    );

    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "Destination Component",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "parent_object": {
                        "base_object": {
                            "foo_string": "deep update",
                            "bar_string": "another update",
                        },
                    },
                },
            }
        ],
        ComponentView::new(ctx, *destination_component.id())
            .await
            .expect("cannot get destination component view")
            .properties,
    );

    // confirm the presence of the correct internal provider value for a leaf of the base_object
    // on the destination component
    let destination_foo_ip = InternalProvider::find_for_prop(ctx, *destination_foo_prop.id())
        .await
        .expect("find ip for foo_string prop")
        .expect("ip for foo string should exist");

    let destination_foo_ip_av = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            internal_provider_id: Some(*destination_foo_ip.id()),
            component_id: Some(*destination_component.id()),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("find attribute value for foo_string internal provider")
    .expect("attribute value for foo_string internal provider should exist");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        Some(serde_json::json!["deep update"]),
        destination_foo_ip_av
            .get_value(ctx)
            .await
            .expect("able to get value")
    );
}
