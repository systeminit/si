use dal::{
    attribute::context::AttributeContextBuilder,
    func::argument::{FuncArgument, FuncArgumentKind},
    job::definition::DependentValuesUpdate,
    provider::internal::InternalProvider,
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue, Component,
    ComponentView, DalContext, ExternalProvider, Func, FuncBackendKind, FuncBackendResponseType,
    PropKind, SocketArity, StandardModel,
};
use dal_test::{
    connection_annotation_string,
    helpers::setup_identity_func,
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn intra_component_identity_update(ctx: &DalContext) {
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "starfield", *schema.id())
            .await
            .expect("unable to create component");

    // This context can also be used for generating component views.
    let base_attribute_read_context = AttributeReadContext {
        prop_id: None,
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that our rendered data matches what was intended.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "destination": "11-nov-2022",
                    "source": "updateme",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
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
    let (identity_func_id, _, _, identity_func_identity_argument_id) =
        setup_identity_func(ctx).await;

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
        identity_func_identity_argument_id,
        *source_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that the shape has not changed after creating the provider and updating the prototype.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "source": "updateme",
                    "destination": "11-nov-2022",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that both the source and destination fields were updated.
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "destination": "h1-2023",
                    "source": "h1-2023",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe it again!
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "starfield",
                "type": "component",
                "protected": false
            },
            "domain": {
                "object": {
                    "destination": "pain.",
                    "source": "pain.",
                },
            },
        }], // expected
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // TODO(nick): add daisy chaining where one field updates another, which in turn, updates
    // another and other kinds of complex updating.
}

#[test]
async fn intra_component_custom_func_update_to_external_provider(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let freya_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "freya",
        PropKind::String,
        *schema_variant.id(),
        Some(root_prop.domain_prop_id),
    )
    .await;
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id, _) =
        setup_identity_func(ctx).await;
    let (external_provider, _output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "freya",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        connection_annotation_string!("freya"),
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create external provider");

    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let freya_provider = InternalProvider::find_for_prop(ctx, *freya_prop.id())
        .await
        .expect("could not execute find for prop for freya")
        .expect("did not find internal provider for freya prop");
    let external_provider_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            external_provider_id: Some(*external_provider.id()),
            ..AttributeReadContext::default()
        },
    )
    .await
    .unwrap()
    .unwrap();

    let mut external_provider_attribute_prototype = external_provider_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not perform get attribute prototype for attribute value")
        .expect("could not find attribute prototype for attribute value");

    // Create and set the func to transform the string field.
    let mut transformation_func = Func::new(
        ctx,
        "test:toUpper",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::String,
    )
    .await
    .expect("could not create func");
    let code = "function toUpper(input) {
        return input.freya.toUpperCase();
    }";
    transformation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    transformation_func
        .set_handler(ctx, Some("toUpper"))
        .await
        .expect("set handler");
    external_provider_attribute_prototype
        .set_func_id(ctx, *transformation_func.id())
        .await
        .expect("set function on attribute prototype for external provider");
    let transformation_func_argument = FuncArgument::new(
        ctx,
        "freya",
        FuncArgumentKind::Object,
        None,
        *transformation_func.id(),
    )
    .await
    .expect("could not create func argument");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype.id(),
        *transformation_func_argument.id(),
        *freya_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    let (component, _) = Component::new(ctx, "valkyrie-queen", *schema_variant.id())
        .await
        .expect("unable to create component");

    let mut base_attribute_context = AttributeContext::builder();
    base_attribute_context.set_component_id(*component.id());

    let domain_context = base_attribute_context
        .clone()
        .set_prop_id(root_prop.domain_prop_id)
        .to_context()
        .expect("cannot create domain context");

    let domain_value = AttributeValue::find_for_context(ctx, domain_context.into())
        .await
        .expect("cannot get domain av")
        .expect("domain av not found");

    let freya_context = base_attribute_context
        .clone()
        .set_prop_id(*freya_prop.id())
        .to_context()
        .expect("cannot create freya write context");

    let freya_value = AttributeValue::find_for_context(ctx, freya_context.into())
        .await
        .expect("cannot get freya av")
        .expect("freya av not found");

    let (_, freya_value_id) = AttributeValue::update_for_context(
        ctx,
        *freya_value.id(),
        Some(*domain_value.id()),
        freya_context,
        Some(serde_json::to_value("for asgard").expect("create json string")),
        None,
    )
    .await
    .expect("run update for context");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let base_attribute_read_context = AttributeReadContext {
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    let external_provider_av = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            external_provider_id: Some(*external_provider.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("could not run find for external provider av")
    .expect("no external provider av");

    assert_eq!(
        Some(serde_json::json!["FOR ASGARD"]),
        external_provider_av
            .get_value(ctx)
            .await
            .expect("get value for external provider av")
    );

    // Now let's update it via a func and ensure the transformation func also runs
    let mut func = Func::new(
        ctx,
        "test:odin",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::String,
    )
    .await
    .expect("could not create func");
    let code = "function odin(_args) {
        return 'odin';
    }";
    func.set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    func.set_handler(ctx, Some("odin"))
        .await
        .expect("set handler");

    let mut freya_value = AttributeValue::get_by_id(ctx, &freya_value_id)
        .await
        .expect("get freya value by id")
        .expect("freya value by id not found");

    let mut freya_prototype = freya_value
        .attribute_prototype(ctx)
        .await
        .expect("get prototype for freya value")
        .expect("prototype for freya value not found");

    freya_prototype
        .set_func_id(ctx, *func.id())
        .await
        .expect("set attribute prototype func");
    freya_value
        .update_from_prototype_function(ctx)
        .await
        .expect("update from proto func");
    ctx.enqueue_job(DependentValuesUpdate::new(
        ctx.access_builder(),
        *ctx.visibility(),
        vec![*freya_value.id()],
    ))
    .await
    .expect("failed to enqueue job");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let external_provider_av = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            external_provider_id: Some(*external_provider.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("could not run find for external provider av")
    .expect("no external provider av");

    assert_eq!(
        Some(serde_json::json!["ODIN"]),
        external_provider_av
            .get_value(ctx)
            .await
            .expect("get value for external provider av")
    );
}
