use dal::attribute::context::AttributeContextBuilder;
use dal::builtins::helpers::setup_identity_func;
use dal::test::helpers::ComponentPayload;
use dal::test_harness::{
    create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant_with_root,
};
use dal::{
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, Connection, DalContext,
    ExternalProvider, InternalProvider, PropKind, SchemaKind, StandardModel,
};
use dal::{Component, SchemaVariant};
use pretty_assertions_sorted::assert_eq_sorted;
use std::collections::HashMap;

use crate::dal::test;

#[test]
async fn inter_component_identity_update(ctx: &DalContext<'_, '_>) {
    // Setup both components used for inter component identity update.
    let esp_payload = setup_esp(ctx).await;
    let swings_payload = setup_swings(ctx).await;

    // Ensure that they look as we expect.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "intermediate": "zero",
                    "source": "zero",
                },
            },
            "si": {
                "name": "esp",
            },
        }], // expected
        esp_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "destination": "zero",
            },
            "si": {
                "name": "swings",
            },
        }], // expected
        swings_payload.component_view_properties(ctx).await // actual
    );

    // Collect the identity func information we need.
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) =
        setup_identity_func(ctx)
            .await
            .expect("could not setup identity func");

    // Setup the "esp" intra component update functionality from "source" to "intermediate".
    let intermediate_attribute_value = AttributeValue::find_for_context(
        ctx,
        esp_payload.attribute_read_context_with_prop_id("/root/domain/object/intermediate"),
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
    let source_internal_provider =
        InternalProvider::get_for_prop(ctx, esp_payload.get_prop_id("/root/domain/object/source"))
            .await
            .expect("could not get internal provider")
            .expect("internal provider not found");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *intermediate_attribute_prototype.id(),
        "identity".to_string(),
        *source_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Update the "esp" field, "source", to see if the intra component connection works.
    let object_attribute_value = AttributeValue::find_for_context(
        ctx,
        esp_payload.attribute_read_context_with_prop_id("/root/domain/object"),
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");
    let source_attribute_value = AttributeValue::find_for_context(
        ctx,
        esp_payload.attribute_read_context_with_prop_id("/root/domain/object/source"),
    )
    .await
    .expect("cannot find attribute value")
    .expect("attribute value not found");
    let (_, updated_source_attribute_value_id, _) = AttributeValue::update_for_context(
        ctx,
        *source_attribute_value.id(),
        Some(*object_attribute_value.id()),
        esp_payload.attribute_context_with_prop_id("/root/domain/object/source"),
        Some(serde_json::to_value("one").expect("could not convert to serde_json::Value")),
        None,
    )
    .await
    .expect("could not update attribute value");

    // Create the "esp" external provider for inter component connection.
    let esp_external_provider = ExternalProvider::new(
        ctx,
        esp_payload.schema_id,
        esp_payload.schema_variant_id,
        "esp".to_string(),
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
    )
    .await
    .expect("could not create external provider");
    let esp_intermediate_internal_provider = InternalProvider::get_for_prop(
        ctx,
        esp_payload.get_prop_id("/root/domain/object/intermediate"),
    )
    .await
    .expect("could not get internal provider")
    .expect("internal provider not found");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *esp_external_provider
            .attribute_prototype_id()
            .expect("no attribute prototype id for external provider"),
        "identity".to_string(),
        *esp_intermediate_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Create the "swings" explicit internal provider for intra component connection.
    let swings_destination_attribute_value = AttributeValue::find_for_context(
        ctx,
        swings_payload.attribute_read_context_with_prop_id("/root/domain/destination"),
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
    let swings_explicit_internal_provider = InternalProvider::new_explicit(
        ctx,
        swings_payload.schema_id,
        swings_payload.schema_variant_id,
        "swings".to_string(),
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
    )
    .await
    .expect("could not create explicit internal provider");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *swings_destination_attribute_prototype.id(),
        "identity".to_string(),
        *swings_explicit_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Ensure that both components look as we expect when not "connected". The creation of both the
    // "esp" external provider and the "swings" implicit internal provider should not affect intra
    // component identity update working.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "intermediate": "one",
                    "source": "one",
                },
            },
            "si": {
                "name": "esp",
            },
        }], // expected
        esp_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "destination": "zero",
            },
            "si": {
                "name": "swings",
            },
        }], // expected
        swings_payload.component_view_properties(ctx).await // actual
    );

    // Connect the two components.
    Connection::connect_providers(
        ctx,
        "identity".to_string(),
        *esp_external_provider.id(),
        esp_payload.component_id,
        *swings_explicit_internal_provider.id(),
        swings_payload.component_id,
    )
    .await
    .expect("could not connect providers");

    // Ensure that both components continue to look as we expect.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "intermediate": "one",
                    "source": "one",
                },
            },
            "si": {
                "name": "esp",
            },
        }], // expected
        esp_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "destination": "zero",
            },
            "si": {
                "name": "swings",
            },
        }], // expected
        swings_payload.component_view_properties(ctx).await // actual
    );

    // Update the "esp" field, "source", again.
    AttributeValue::update_for_context(
        ctx,
        updated_source_attribute_value_id,
        Some(*object_attribute_value.id()),
        esp_payload.attribute_context_with_prop_id("/root/domain/object/source"),
        Some(serde_json::to_value("two").expect("could not convert to serde_json::Value")),
        None,
    )
    .await
    .expect("could not update attribute value");

    // Observe that inter component identity updating work.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "intermediate": "two",
                    "source": "two",
                },
            },
            "si": {
                "name": "esp",
            },
        }], // expected
        esp_payload.component_view_properties(ctx).await // actual
    );
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "destination": "two",
            },
            "si": {
                "name": "swings",
            },
        }], // expected
        swings_payload.component_view_properties(ctx).await // actual
    );
}

// 38.805354552534816, -77.05091482877533
async fn setup_esp(ctx: &DalContext<'_, '_>) -> ComponentPayload {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // "esp"
    // domain: Object
    // └─ object: Object
    //    ├─ source: String
    //    └─ intermediate: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root_prop.domain_prop_id,
    )
    .await;
    let source_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "source",
        *object_prop.id(),
    )
    .await;
    let intermediate_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "intermediate",
        *object_prop.id(),
    )
    .await;
    let mut prop_map = HashMap::new();
    prop_map.insert("/root/domain/object", *object_prop.id());
    prop_map.insert("/root/domain/object/source", *source_prop.id());
    prop_map.insert("/root/domain/object/intermediate", *intermediate_prop.id());

    // Create the internal providers for a schema variant. Afterwards, we can create the component.
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("could not create internal providers for schema variant");
    let (component, _, _) = Component::new_for_schema_with_node(ctx, "esp", schema.id())
        .await
        .expect("unable to create component");

    // This context can also be used for generating component views.
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    // Initialize the value corresponding to the "source" prop.
    let unset_object_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*object_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");
    let source_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*source_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");
    let source_prop_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*source_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let value = serde_json::to_value("zero").expect("could not convert to serde_json::Value");
    AttributeValue::update_for_context(
        ctx,
        *source_attribute_value.id(),
        Some(*unset_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot update value for context");

    // Initialize the value corresponding to the "intermediate" prop.
    let set_object_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*object_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");
    let intermediate_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*intermediate_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");
    let intermediate_prop_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*intermediate_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let value = serde_json::to_value("zero").expect("could not convert to serde_json::Value");
    AttributeValue::update_for_context(
        ctx,
        *intermediate_attribute_value.id(),
        Some(*set_object_attribute_value.id()),
        intermediate_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot set value for context");

    // Return the payload.
    ComponentPayload {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        component_id: *component.id(),
        prop_map,
        base_attribute_read_context,
    }
}

// 38.82091849697006, -77.05236860190759
async fn setup_swings(ctx: &DalContext<'_, '_>) -> ComponentPayload {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // "swings"
    // domain: Object
    // └─ destination: string
    let destination_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "destination",
        root_prop.domain_prop_id,
    )
    .await;
    let mut prop_map = HashMap::new();
    prop_map.insert("/root/domain/destination", *destination_prop.id());

    // Create the internal providers for a schema variant. Afterwards, we can create the component.
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("could not create internal providers for schema variant");
    let (component, _, _) = Component::new_for_schema_with_node(ctx, "swings", schema.id())
        .await
        .expect("unable to create component");

    // This context can also be used for generating component views.
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    // Initialize the value corresponding to the "destination" prop.
    let domain_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root_prop.domain_prop_id),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");
    let destination_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*destination_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");
    let destination_prop_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*destination_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");
    let value = serde_json::to_value("zero").expect("could not convert to serde_json::Value");
    AttributeValue::update_for_context(
        ctx,
        *destination_attribute_value.id(),
        Some(*domain_attribute_value.id()),
        destination_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot update value for context");

    // Return the payload.
    ComponentPayload {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        component_id: *component.id(),
        prop_map,
        base_attribute_read_context,
    }
}
