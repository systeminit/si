use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::schema::variant::leaves::LeafKind;
use dal::SchemaVariant;
use dal::{
    attribute::context::AttributeContextBuilder, AttributeReadContext, AttributeValue, Component,
    ComponentView, DalContext, Func, FuncBackendKind, FuncBackendResponseType, PropKind,
    SchemaKind, StandardModel,
};
use dal_test::test_harness::create_prop_and_set_parent;
use dal_test::{
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn check_js_validation_for_component(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let prop =
        create_prop_and_set_parent(ctx, PropKind::String, "Tamarian", root_prop.domain_prop_id)
            .await;

    let mut func = Func::new(
        ctx,
        "test:jsValidation",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("create js validation func");

    let js_validation_code = "function validate(input) {
        return {
            valid: (input?.value ?? '') === 'Temba, his arms open', message: 'Darmok and Jalad at Tanagra'
        };
    }";

    func.set_code_plaintext(ctx, Some(js_validation_code))
        .await
        .expect("set code");
    func.set_handler(ctx, Some("validate"))
        .await
        .expect("set handler");
    let func_arg = FuncArgument::new(ctx, "value", FuncArgumentKind::String, None, *func.id())
        .await
        .expect("create validation func arg");

    schema_variant
        .finalize(ctx)
        .await
        .expect("could not finalize");

    let tamarian_prop_ip = prop
        .internal_provider(ctx)
        .await
        .expect("could not get ip for Tamarian prop")
        .expect("ip for tamarian prop not found");

    let (_, map_key) = SchemaVariant::add_leaf(
        ctx,
        *func.id(),
        *func_arg.id(),
        *tamarian_prop_ip.id(),
        *schema_variant.id(),
        LeafKind::Validation,
    )
    .await
    .expect("cannot add validation leaf to schema_variant");

    let (component, _) =
        Component::new_for_schema_variant_with_node(ctx, "Danoth", schema_variant.id())
            .await
            .expect("could not create component");

    let base_attribute_read_context = AttributeReadContext {
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    let av = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("could not find attribute value");

    let pav = av
        .parent_attribute_value(ctx)
        .await
        .expect("could not get parent attribute value");

    let av_update_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*prop.id())
        .to_context()
        .expect("make attribute context for update");

    let (_, updated_av_id) = AttributeValue::update_for_context(
        ctx,
        *av.id(),
        pav.map(|pav| *pav.id()),
        av_update_context,
        Some(serde_json::json!("Shaka, when the walls fell")),
        None,
    )
    .await
    .expect("update attr value");

    let properties = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view")
        .properties;

    assert_eq!(
        serde_json::json!({
            "si": {
                "name": "Danoth",
            },

            "domain": {
                "Tamarian": "Shaka, when the walls fell",
            },

            "validation": {
                map_key.clone(): {
                    "valid": false,
                    "message": "Darmok and Jalad at Tanagra",
                }
            }
        }),
        properties
    );

    let validations = Component::list_validations(ctx, *component.id())
        .await
        .expect("got validations");
    assert_eq!(1, validations.len());
    assert_eq!(*prop.id(), validations[0].prop_id);
    assert_eq!(false, validations[0].valid);
    assert_eq!(
        Some("Darmok and Jalad at Tanagra".to_string()),
        validations[0].message
    );

    // Change the function code, re-execute the function and ensure we get back just the
    // latest validation
    let js_validation_code = "function validate(input) {
        return {
            valid: input.value === 'Temba, his arms open', message: 'Darmok and Jalad on the ocean'
        };
    }";

    func.set_code_plaintext(ctx, Some(js_validation_code))
        .await
        .expect("Update validation func code");

    let av = AttributeValue::get_by_id(ctx, &updated_av_id)
        .await
        .expect("get updated av")
        .expect("not none");
    let pav = av
        .parent_attribute_value(ctx)
        .await
        .expect("could not get parent attribute value");

    let (_, _updated_av_id) = AttributeValue::update_for_context(
        ctx,
        *av.id(),
        pav.map(|pav| *pav.id()),
        av_update_context,
        Some(serde_json::json!("Temba, his arms open")),
        None,
    )
    .await
    .expect("update attr value");

    let properties = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view")
        .properties;

    assert_eq!(
        serde_json::json!({
            "si": {
                "name": "Danoth",
            },

            "domain": {
                "Tamarian": "Temba, his arms open",
            },

            "validation": {
                map_key: {
                    "valid": true,
                    "message": "Darmok and Jalad on the ocean",
                }
            }

        }),
        properties
    );

    let validations = Component::list_validations(ctx, *component.id())
        .await
        .expect("got validations");
    assert_eq!(1, validations.len());
    assert_eq!(*prop.id(), validations[0].prop_id);
    assert_eq!(true, validations[0].valid);
    assert_eq!(
        Some("Darmok and Jalad on the ocean".to_string()),
        validations[0].message
    );
}
