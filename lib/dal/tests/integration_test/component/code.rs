use dal::attribute::context::AttributeContextBuilder;
use dal::func::argument::FuncArgument;
use dal::{
    AttributeReadContext, AttributeValue, CodeLanguage, Component, ComponentView, DalContext, Func,
    PropKind, SchemaKind, SchemaVariant, StandardModel,
};
use dal_test::test;
use dal_test::test_harness::{
    create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant_with_root,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn set_code_prop_for_component(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // domain: Object
    // └─ poop: String
    let poop_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "poop",
        root_prop.domain_prop_id,
    )
    .await;

    // Create code prototype(s).
    let func_name = "si:generateYAML".to_owned();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");
    let code_generation_func_argument =
        FuncArgument::find_by_name_for_func(ctx, "domain", *func.id())
            .await
            .expect("could not perform func argument find")
            .expect("no func argument found");

    SchemaVariant::add_code_generation(
        ctx,
        *func.id(),
        *code_generation_func_argument.id(),
        *schema_variant.id(),
        CodeLanguage::Yaml,
    )
    .await
    .expect("could not add code generation");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx)
        .await
        .expect("unable to finalize schema variant");

    let (component, _) =
        Component::new_for_schema_variant_with_node(ctx, "component", schema_variant.id())
            .await
            .expect("cannot create component");

    // Set a value on the prop to check if our code generation works as intended.
    let read_context = AttributeReadContext {
        prop_id: Some(*poop_prop.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let attribute_value = AttributeValue::find_for_context(ctx, read_context)
        .await
        .expect("could not perform find for context")
        .expect("attribute value not found");
    let parent_attribute_value = attribute_value
        .parent_attribute_value(ctx)
        .await
        .expect("could not perform find parent attribute value")
        .expect("no parent attribute value found");
    let context = AttributeContextBuilder::from(read_context)
        .to_context()
        .expect("could not convert builder to attribute context");
    AttributeValue::update_for_context(
        ctx,
        *attribute_value.id(),
        Some(*parent_attribute_value.id()),
        context,
        Some(serde_json::json!["canoe"]),
        None,
    )
    .await
    .expect("could not perform update for context");

    // Observe that the code generation worked.
    let component_view = ComponentView::for_context(
        ctx,
        AttributeReadContext {
            prop_id: None,
            component_id: Some(*component.id()),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("could not generate component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "component",
                },
                "domain": {
                    "poop": "canoe",
                },
                "code": {
                    "si:generateYAML": {
                        "code": "poop: canoe\n",
                        "format": "yaml",
                    },
                }
        }], // expected
        component_view.properties // actual
    );

    // Ensure the code view looks as we expect it to.
    let mut code_views = Component::list_code_generated(ctx, *component.id())
        .await
        .expect("could not list code generated for component");
    let code_view = code_views.pop().expect("code views are empty");
    assert!(code_views.is_empty());
    assert_eq!(CodeLanguage::Yaml, code_view.language);
    assert_eq!(Some("poop: canoe\n".to_string()), code_view.code);
}
