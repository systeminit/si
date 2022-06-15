use crate::dal::test;
use dal::attribute::context::AttributeContextBuilder;
use dal::func::binding::FuncBinding;
use dal::provider::internal::InternalProvider;
use dal::test::helpers::find_prop_and_parent_by_name;
use dal::test_harness::{
    create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant_with_root,
};
use dal::{
    AttributePrototypeArgument, AttributeValue, Component, ComponentView, Schema, SchemaVariant,
};
use dal::{AttributeReadContext, DalContext, Func, PropKind, SchemaKind, StandardModel};
use pretty_assertions_sorted::assert_eq_sorted;

#[test]
async fn intra_component_identity_update(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
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

    // Create the internal providers for a schema variant. Afterwards, we can create the component.
    SchemaVariant::create_implicit_internal_providers(ctx, *schema.id(), *schema_variant.id())
        .await
        .expect("could not create internal providers for schema variant");
    let (component, _, _) = Component::new_for_schema_with_node(ctx, "starfield", schema.id())
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
    let (_, updated_source_attribute_value_id, task) = AttributeValue::update_for_context(
        ctx,
        *source_attribute_value.id(),
        Some(*unset_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot update value for context");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");

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
    let (_, updated_destination_attribute_value_id, task) = AttributeValue::update_for_context(
        ctx,
        *destination_attribute_value.id(),
        Some(*set_object_attribute_value.id()),
        destination_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("cannot set value for context");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");

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
        "identity".to_string(),
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
    let (_, twice_updated_source_attribute_value_id, task) = AttributeValue::update_for_context(
        ctx,
        updated_source_attribute_value_id,
        Some(*set_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("could not update attribute value");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");

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
    let (_, _, task) = AttributeValue::update_for_context(
        ctx,
        twice_updated_source_attribute_value_id,
        Some(*set_object_attribute_value.id()),
        source_prop_context,
        Some(value),
        None,
    )
    .await
    .expect("could not update attribute value");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");

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

#[test]
async fn docker_image_intra_component_update(ctx: &DalContext<'_, '_>) {
    let schema_name = "docker_image".to_string();
    let schema: Schema = Schema::find_by_attr(ctx, "name", &schema_name)
        .await
        .expect("could not find schema by name")
        .pop()
        .expect("schema not found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("default schema variant id not found");

    // Create two components using the docker image schema.
    let (soulrender_component, _, task) =
        Component::new_for_schema_with_node(ctx, "soulrender", schema.id())
            .await
            .expect("unable to create component");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");
    let soulrender_base_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant_id),
        component_id: Some(*soulrender_component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "soulrender"
            },
            "si": {
                "name": "soulrender",
            },
        }], // expected
        ComponentView::for_context(ctx, soulrender_base_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    let (bloodscythe_component, _, task) =
        Component::new_for_schema_with_node(ctx, "bloodscythe", schema.id())
            .await
            .expect("unable to create component");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");
    let bloodscythe_base_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant_id),
        component_id: Some(*bloodscythe_component.id()),
        ..AttributeReadContext::default()
    };

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe"
            },
            "si": {
                "name": "bloodscythe",
            },
        }], // expected
        ComponentView::for_context(ctx, bloodscythe_base_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Update the second component and ensure the first's values did not drift. First, find the prop
    // that we want to update on the schema variant.
    let (name_prop_id, si_prop_id) =
        find_prop_and_parent_by_name(ctx, "name", "si", None, *schema_variant_id)
            .await
            .expect("could not find prop and parent by name");

    // Now, find the attribute values need to update the second component.
    let bloodscythe_si_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(si_prop_id),
            ..bloodscythe_base_context
        },
    )
    .await
    .expect("could not find attribute value for context")
    .expect("attribute value not found");
    let bloodscythe_name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(name_prop_id),
            ..bloodscythe_base_context
        },
    )
    .await
    .expect("could not find attribute value for context")
    .expect("attribute value not found");

    // Update the "/root/si/name" value on the second component, observe that it worked, and observe
    // that the first component was not updated.
    let bloodscythe_name_update_context = AttributeContextBuilder::from(bloodscythe_base_context)
        .set_prop_id(name_prop_id)
        .to_context()
        .expect("could not convert builder to attribute context");
    let (_, _, task) = AttributeValue::update_for_context(
        ctx,
        *bloodscythe_name_attribute_value.id(),
        Some(*bloodscythe_si_attribute_value.id()),
        bloodscythe_name_update_context,
        Some(
            serde_json::to_value("bloodscythe-updated")
                .expect("could not convert to serde_json::Value"),
        ),
        None,
    )
    .await
    .expect("could not update attribute value for context");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe-updated"
            },
            "si": {
                "name": "bloodscythe-updated",
            },
        }], // expected
        ComponentView::for_context(ctx, bloodscythe_base_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "soulrender"
            },
            "si": {
                "name": "soulrender",
            },
        }], // expected
        ComponentView::for_context(ctx, soulrender_base_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    // Let's update the first component too. Like before, find the attribute values need to update
    // the first component.
    let soulrender_si_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(si_prop_id),
            ..soulrender_base_context
        },
    )
    .await
    .expect("could not find attribute value for context")
    .expect("attribute value not found");
    let soulrender_name_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(name_prop_id),
            ..soulrender_base_context
        },
    )
    .await
    .expect("could not find attribute value for context")
    .expect("attribute value not found");

    // Update the "/root/si/name" value on the first component, observe that it worked, and observe
    // that the second component was not updated.
    let soulrender_name_update_context = AttributeContextBuilder::from(soulrender_base_context)
        .set_prop_id(name_prop_id)
        .to_context()
        .expect("could not convert builder to attribute context");
    let (_, _, task) = AttributeValue::update_for_context(
        ctx,
        *soulrender_name_attribute_value.id(),
        Some(*soulrender_si_attribute_value.id()),
        soulrender_name_update_context,
        Some(
            serde_json::to_value("soulrender-updated")
                .expect("could not convert to serde_json::Value"),
        ),
        None,
    )
    .await
    .expect("could not update attribute value for context");
    task.run_updates_in_ctx(&ctx)
        .await
        .expect("unable to run dependent values update");

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "bloodscythe-updated"
            },
            "si": {
                "name": "bloodscythe-updated",
            },
        }], // expected
        ComponentView::for_context(ctx, bloodscythe_base_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );

    assert_eq_sorted!(
        serde_json::json![{
            "domain": {
                "image": "soulrender-updated"
            },
            "si": {
                "name": "soulrender-updated",
            },
        }], // expected
        ComponentView::for_context(ctx, soulrender_base_context)
            .await
            .expect("cannot get component view")
            .properties // actual
    );
}
