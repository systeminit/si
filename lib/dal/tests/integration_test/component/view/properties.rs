use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::schema::variant::leaves::LeafKind;
use dal::{
    attribute::context::AttributeContextBuilder,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    AttributeReadContext, AttributeValue, Component, ComponentView, ComponentViewProperties,
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, PropKind, SchemaVariant,
    StandardModel,
};

use dal_test::{
    test,
    test_harness::{create_prop_and_set_parent, create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn drop_subtree_using_component_view_properties(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    let schema_variant_id = *schema_variant.id();
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await
        .expect("cannot set default schema variant");
    let poop_prop =
        create_prop_and_set_parent(ctx, PropKind::String, "poop", root_prop.domain_prop_id).await;

    // Create a fake code gen func.
    let mut code_generation_func = Func::new(
        ctx,
        "test:codeGeneration",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::CodeGeneration,
    )
    .await
    .expect("could not create func");
    let code_generation_func_id = *code_generation_func.id();
    let code = "function generate(input) {
        return {
            code: input.domain?.poop,
            format: \"json\"
        };
    }";
    code_generation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    code_generation_func
        .set_handler(ctx, Some("generate"))
        .await
        .expect("set handler");
    let code_generation_func_argument = FuncArgument::new(
        ctx,
        "domain",
        FuncArgumentKind::Object,
        None,
        code_generation_func_id,
    )
    .await
    .expect("could not create func argument");

    // Add a code generation leaf.
    SchemaVariant::add_leaf(
        ctx,
        code_generation_func_id,
        schema_variant_id,
        None,
        LeafKind::CodeGeneration,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *code_generation_func_argument.id(),
        }],
    )
    .await
    .expect("could not add code generation");

    // Finalize the variant and create a component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");
    let (component, _) = Component::new(ctx, "component", schema_variant_id)
        .await
        .expect("cannot create component");

    // Check the view and properties before updating the poop field.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not create component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "code": {
                "test:codeGeneration": {}
            },
            "domain": {},
        }], // expected
        component_view.properties // actual
    );

    let mut component_view_properties = ComponentViewProperties::try_from(component_view)
        .expect("could not convert component view to component view properties");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {}
        }], // expected
        component_view_properties
            .drop_code()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Update the poop field, which will cause the code generation entry to be updated.
    let poop_attribute_read_context = AttributeReadContext {
        prop_id: Some(*poop_prop.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let poop_attribute_context = AttributeContextBuilder::from(poop_attribute_read_context)
        .to_context()
        .expect("could not convert builder to context");
    let poop_attribute_value = AttributeValue::find_for_context(ctx, poop_attribute_read_context)
        .await
        .expect("could not perform find for context")
        .expect("attribute value not found");
    let domain_attribute_value = poop_attribute_value
        .parent_attribute_value(ctx)
        .await
        .expect("could not perform parent attribute value")
        .expect("parent attribute value not found");
    AttributeValue::update_for_context(
        ctx,
        *poop_attribute_value.id(),
        Some(*domain_attribute_value.id()),
        poop_attribute_context,
        Some(serde_json::json!["canoe"]),
        None,
    )
    .await
    .expect("could not update for context");

    // Check the value with and without the code subtree.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not create component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "code": {
                "test:codeGeneration": {
                    "code": "canoe",
                    "format": "json",
                }
            },
            "domain": {
                "poop": "canoe"
            },
        }], // expected
        component_view.properties // actual
    );

    let mut component_view_properties = ComponentViewProperties::try_from(component_view)
        .expect("could not convert component view to component view properties");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "protected": false
            },
            "domain": {
                "poop": "canoe"
            }
        }], // expected
        component_view_properties
            .drop_code()
            .to_value()
            .expect("could not convert to value") // actual
    );
}
