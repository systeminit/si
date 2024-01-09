use pretty_assertions_sorted::assert_eq;

use dal::component::ComponentKind;
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::schema::variant::leaves::LeafKind;
use dal::{
    attribute::context::AttributeContextBuilder,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    FuncBackendKind, FuncBackendResponseType, Prop, Schema,
};
use dal::{
    AttributeReadContext, AttributeValue, CodeLanguage, Component, ComponentView, DalContext, Func,
    PropKind, SchemaVariant, StandardModel,
};
use dal_test::test;
use dal_test::test_harness::{create_schema, create_schema_variant_with_root};

#[test]
async fn add_code_generation_and_list_code_views(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    // domain: Object
    // └─ poop: String
    let poop_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "poop",
        PropKind::String,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await;
    // Create code prototype(s).
    let mut func = Func::new(
        ctx,
        "test:codeGeneration",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::CodeGeneration,
    )
    .await
    .expect("could not create func");
    let code = "function generateYAML(input) {
      return {
        format: \"yaml\",
        code: Object.keys(input.domain).length > 0 ? YAML.stringify(input.domain) : \"\"
      };
    }";
    func.set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    func.set_handler(ctx, Some("generateYAML"))
        .await
        .expect("set handler");
    let func_argument =
        FuncArgument::new(ctx, "domain", FuncArgumentKind::Object, None, *func.id())
            .await
            .expect("could not create func argument");

    SchemaVariant::add_leaf(
        ctx,
        *func.id(),
        *schema_variant.id(),
        None,
        LeafKind::CodeGeneration,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *func_argument.id(),
        }],
    )
    .await
    .expect("could not add code generation");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) = Component::new(ctx, "component", *schema_variant.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that the code generation worked.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "component",
                    "type": "component",
                    "protected": false,
                },
                "code": {
                    "test:codeGeneration": {
                        "code": "poop: canoe\n",
                        "format": "yaml",
                    },
                },
                "domain": {
                    "poop": "canoe",
                }
        }], // expected
        component_view.properties // actual
    );

    // Ensure the code view looks as we expect it to.
    let (mut code_views, _) = Component::list_code_generated(ctx, *component.id())
        .await
        .expect("could not list code generated for component");
    let code_view = code_views.pop().expect("code views are empty");
    assert!(code_views.is_empty());
    assert_eq!(CodeLanguage::Yaml, code_view.language);
    assert_eq!(Some("poop: canoe\n".to_string()), code_view.code);
}

#[test]
async fn all_code_generation_attribute_values(ctx: &DalContext) {
    // Create two schemas and variants.
    let mut navi_schema = Schema::new(ctx, "navi", &ComponentKind::Standard)
        .await
        .expect("cannot create schema");
    let (mut navi_schema_variant, navi_root_prop) =
        create_schema_variant_with_root(ctx, *navi_schema.id()).await;
    navi_schema
        .set_default_schema_variant_id(ctx, Some(*navi_schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let ange1_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "ange1",
        PropKind::String,
        *navi_schema_variant.id(),
        Some(navi_root_prop.domain_prop_id),
    )
    .await;
    let mut kru_schema = Schema::new(ctx, "kru", &ComponentKind::Standard)
        .await
        .expect("cannot create schema");
    let (mut kru_schema_variant, kru_root_prop) =
        create_schema_variant_with_root(ctx, *kru_schema.id()).await;
    kru_schema
        .set_default_schema_variant_id(ctx, Some(*kru_schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let _melser_prop = dal_test::test_harness::create_prop_without_ui_optionals(
        ctx,
        "melser",
        PropKind::String,
        *kru_schema_variant.id(),
        Some(kru_root_prop.domain_prop_id),
    )
    .await;
    // Create two code generation funcs.
    let code = "function generateYAML(input) {
      return {
        format: \"yaml\",
        code: Object.keys(input.domain).length > 0 ? YAML.stringify(input.domain) : \"\"
      };
    }";

    let mut func_one = Func::new(
        ctx,
        "test:codeGenerationOne",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::CodeGeneration,
    )
    .await
    .expect("could not create func");
    func_one
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    func_one
        .set_handler(ctx, Some("generateYAML"))
        .await
        .expect("set handler");
    let func_one_domain_argument = FuncArgument::new(
        ctx,
        "domain",
        FuncArgumentKind::Object,
        None,
        *func_one.id(),
    )
    .await
    .expect("could not create func argument");

    let mut func_two = Func::new(
        ctx,
        "test:codeGenerationTwo",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::CodeGeneration,
    )
    .await
    .expect("could not create func");
    func_two
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    func_two
        .set_handler(ctx, Some("generateYAML"))
        .await
        .expect("set handler");
    let func_two_domain_argument = FuncArgument::new(
        ctx,
        "domain",
        FuncArgumentKind::Object,
        None,
        *func_two.id(),
    )
    .await
    .expect("could not create func argument");

    // Add two leaves to one variant and one leaf of the same func as one of prior to the other.
    SchemaVariant::add_leaf(
        ctx,
        *func_one.id(),
        *navi_schema_variant.id(),
        None,
        LeafKind::CodeGeneration,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *func_one_domain_argument.id(),
        }],
    )
    .await
    .expect("could not add code generation");
    SchemaVariant::add_leaf(
        ctx,
        *func_two.id(),
        *navi_schema_variant.id(),
        None,
        LeafKind::CodeGeneration,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *func_two_domain_argument.id(),
        }],
    )
    .await
    .expect("could not add code generation");
    SchemaVariant::add_leaf(
        ctx,
        *func_one.id(),
        *kru_schema_variant.id(),
        None,
        LeafKind::CodeGeneration,
        vec![LeafInput {
            location: LeafInputLocation::Domain,
            func_argument_id: *func_one_domain_argument.id(),
        }],
    )
    .await
    .expect("could not add code generation");

    // Finalize both variants and create three components.
    navi_schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");
    kru_schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (navi_component, _) = Component::new(ctx, "navi", *navi_schema_variant.id())
        .await
        .expect("cannot create component");
    let (_kru_one_component, _) = Component::new(ctx, "kru-one", *kru_schema_variant.id())
        .await
        .expect("cannot create component");
    let (_kru_two_component, _) = Component::new(ctx, "kru-two", *kru_schema_variant.id())
        .await
        .expect("cannot create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Test our function and perform assertions on the results.
    check_results(ctx).await;

    // Update a value and check the component view.
    let attribute_read_context = AttributeReadContext {
        prop_id: Some(*ange1_prop.id()),
        component_id: Some(*navi_component.id()),
        ..AttributeReadContext::default()
    };
    let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
        .await
        .expect("could not perform find for context")
        .expect("attribute value not found");
    let parent_attribute_value = attribute_value
        .parent_attribute_value(ctx)
        .await
        .expect("could not perform find parent attribute value")
        .expect("no parent attribute value found");
    let context = AttributeContextBuilder::from(attribute_read_context)
        .to_context()
        .expect("could not convert builder to attribute context");
    AttributeValue::update_for_context(
        ctx,
        *attribute_value.id(),
        Some(*parent_attribute_value.id()),
        context,
        Some(serde_json::json!["omen"]),
        None,
    )
    .await
    .expect("could not perform update for context");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let component_view = ComponentView::new(ctx, *navi_component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "navi",
                    "type": "component",
                    "protected": false,
                },
                "code": {
                    "test:codeGenerationOne": {
                        "code": "ange1: omen\n",
                        "format": "yaml",
                    },
                    "test:codeGenerationTwo": {
                        "code": "ange1: omen\n",
                        "format": "yaml",
                    },
                },
                "domain": {
                    "ange1": "omen",
                }
        }], // expected
        component_view.properties // actual
    );

    // Finally, check the results again to ensure that they didn't drift.
    check_results(ctx).await;
}

async fn check_results(ctx: &DalContext) {
    let all_values = Component::all_code_generation_attribute_values(ctx)
        .await
        .expect("could not get all code generation attribute values");
    assert_eq!(
        4,                // expected
        all_values.len(), // actual
    );

    let mut found_navi_code_generation_one = false;
    let mut found_navi_code_generation_two = false;
    let mut found_kru_one_code_generation_one = false;
    let mut found_kru_two_code_generation_one = false;
    for id in all_values {
        let attribute_value = AttributeValue::get_by_id(ctx, &id)
            .await
            .expect("could not perform get by id")
            .expect("attribute value not found");
        let key = attribute_value.key.expect("key not found");
        let code_item_prop = Prop::get_by_id(ctx, &attribute_value.context.prop_id())
            .await
            .expect("could not perform get by id")
            .expect("prop not found");
        assert_eq!(
            "codeItem",            // expected
            code_item_prop.name(), // actual
        );
        let component = Component::get_by_id(ctx, &attribute_value.context.component_id())
            .await
            .expect("could not perform get by id")
            .expect("component not found");
        let component_name = component.name(ctx).await.expect("could not get name");

        if "test:codeGenerationOne" == &key && "navi" == &component_name {
            assert!(!found_navi_code_generation_one);
            found_navi_code_generation_one = true;
        } else if "test:codeGenerationTwo" == &key && "navi" == &component_name {
            assert!(!found_navi_code_generation_two);
            found_navi_code_generation_two = true;
        } else if "test:codeGenerationOne" == &key && "kru-one" == &component_name {
            assert!(!found_kru_one_code_generation_one);
            found_kru_one_code_generation_one = true;
        } else if "test:codeGenerationOne" == &key && "kru-two" == &component_name {
            assert!(!found_kru_two_code_generation_one);
            found_kru_two_code_generation_one = true;
        }
    }

    // Ensure that we found every code generation. We ensured that we could only find them exactly
    // once in the loop above, but now we need to ensure that we found them at all.
    assert!(found_navi_code_generation_one);
    assert!(found_navi_code_generation_two);
    assert!(found_kru_one_code_generation_one);
    assert!(found_kru_two_code_generation_one);
}
