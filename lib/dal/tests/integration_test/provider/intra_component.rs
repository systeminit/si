use crate::dal::test;
use dal::attribute::context::AttributeContextBuilder;
use dal::func::binding::FuncBinding;
use dal::provider::internal::InternalProvider;
use dal::test_harness::{
    create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant_with_root,
};
use dal::{AttributePrototypeArgument, AttributeValue, Component, ComponentView};
use dal::{AttributeReadContext, DalContext, Func, PropKind, SchemaKind, StandardModel};
use pretty_assertions_sorted::assert_eq_sorted;

#[test]
async fn intra_component_identity_update(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // domain: Object
    // └─ object: Object
    //    ├─ source: String
    //    └─ destination: String
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
    let destination_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "destination",
        *object_prop.id(),
    )
    .await;

    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    let (component, _) = Component::new_for_schema_with_node(ctx, "starfield", schema.id())
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
    let value = serde_json::to_value("updateme").expect("could not convert to serde_json::Value");
    let (_, updated_source_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        *source_attribute_value.id(),
        Some(*unset_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot update value for context");

    // Initialize the value corresponding to the "destination" prop.
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
    let value =
        serde_json::to_value("11-nov-2022").expect("could not convert to serde_json::Value");
    let (_, updated_destination_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        *destination_attribute_value.id(),
        Some(*set_object_attribute_value.id()),
        destination_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot set value for context");

    // Ensure that our rendered data matches what was intended.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "destination": "11-nov-2022",
                    "source": "updateme",
                },
            },
            "si": {
                "name": "starfield",
            },
        }], // expected
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Find the prototype corresponding to the "destination" value (that corresponds to the
    // "destination" prop. Assemble what we need to update the "destination" prototype to use the
    // identity function.
    let updated_destination_attribute_value =
        AttributeValue::get_by_id(ctx, &updated_destination_attribute_value_id)
            .await
            .expect("cannot find attribute value")
            .expect("attribute value not found");
    let mut destination_attribute_prototype = updated_destination_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("cannot find attribute prototype")
        .expect("attribute prototype not found");
    let identity_func: Func = Func::find_by_attr(ctx, "name", &"si:identity".to_string())
        .await
        .expect("could not find func by name attr")
        .pop()
        .expect("identity func not found");
    let (_identity_func_binding, _identity_func_binding_return_value) =
        FuncBinding::find_or_create_and_execute(
            ctx,
            serde_json::json![{ "identity": null }],
            *identity_func.id(),
        )
        .await
        .expect("could not find or create identity func binding");

    // Now, update the "destination" field's corresponding prototype to use the identity function
    // and the source internal provider.
    let source_internal_provider = InternalProvider::get_for_prop(ctx, *source_prop.id())
        .await
        .expect("could not get internal provider")
        .expect("internal provider not found");
    destination_attribute_prototype
        .set_func_id(ctx, *identity_func.id())
        .await
        .expect("could not set func id on attribute prototype");

    // With the "source" internal provider in hand and the "destination" attribute prototype setup,
    // we can create an argument for the latter prototype.
    let _argument = AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *destination_attribute_prototype.id(),
        "identity",
        *source_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Ensure that the shape has not changed after creating the provider and updating the prototype.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "destination": "11-nov-2022",
                    "source": "updateme",
                },
            },
            "si": {
                "name": "starfield",
            },
        }], // expected
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Update the source field.
    let value = serde_json::to_value("h1-2023").expect("could not convert to serde_json::Value");
    let (_, twice_updated_source_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        updated_source_attribute_value_id,
        Some(*set_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("could not update attribute value");

    // Observe that both the source and destination fields were updated.
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "destination": "h1-2023",
                    "source": "h1-2023",
                },
            },
            "si": {
                "name": "starfield",
            },
        }], // expected
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Update it again!
    let value = serde_json::to_value("pain.").expect("could not convert to serde_json::Value");
    let (_, _) = AttributeValue::update_for_context(
        ctx,
        twice_updated_source_attribute_value_id,
        Some(*set_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("could not update attribute value");

    // Observe it again!
    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "object": {
                    "destination": "pain.",
                    "source": "pain.",
                },
            },
            "si": {
                "name": "starfield",
            },
        }], // expected
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // TODO(nick): add daisy chaining where one field updates another, which in turn, updates
    // another and other kinds of complex updating.
}
