use dal::{
    attribute::context::AttributeContextBuilder,
    func::backend::validation::FuncBackendValidationArgs,
    validation::{Validation, ValidationError, ValidationErrorKind},
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentView, DalContext,
    Func, FuncBackendKind, FuncBackendResponseType, PropId, PropKind, StandardModel,
    ValidationPrototype, ValidationPrototypeContext, ValidationResolver, ValidationStatus,
};
use dal_test::test_harness::create_prop_and_set_parent;
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;
use std::collections::HashMap;

#[test]
async fn check_validations_for_component(ctx: &DalContext) {
    // Setup the schema and schema variant and create a validation for a string field.
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let gecs_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "thousand_gecs",
        root_prop.domain_prop_id,
    )
    .await;
    let prefix_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "the_tree_of_clues",
        root_prop.domain_prop_id,
    )
    .await;

    // Gather what we need to create validations
    let mut builder = ValidationPrototypeContext::builder();
    builder.set_schema_id(*schema.id());
    builder.set_schema_variant_id(*schema_variant.id());
    let func_name = "si:validation".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("could not perform find by attr");
    let func = funcs.pop().expect("could not find func");

    // Match validation
    builder.set_prop_id(*gecs_prop.id());
    let args = serde_json::to_value(FuncBackendValidationArgs::new(Validation::StringEquals {
        value: None,
        expected: "stupidHorse".to_string(),
    }))
    .expect("could not convert args to Value");
    ValidationPrototype::new(
        ctx,
        *func.id(),
        args,
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("could not create validation prototype");

    // Prefix validation
    builder.set_prop_id(*prefix_prop.id());
    let args = serde_json::to_value(FuncBackendValidationArgs::new(
        Validation::StringHasPrefix {
            value: None,
            expected: "tooth".to_string(),
        },
    ))
    .expect("could not convert args to Value");
    ValidationPrototype::new(
        ctx,
        *func.id(),
        args,
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("could not create validation prototype");

    // Finalize schema
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize SchemaVariant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Once setup is complete, create a component and update the target field to an "invalid" value.
    let (component, _) = Component::new(ctx, "hundo_gecs", *schema_variant.id())
        .await
        .expect("could not create component");

    let base_attribute_read_context = AttributeReadContext {
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };

    let gecs_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*gecs_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("could not find attribute value");

    let domain_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root_prop.domain_prop_id),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("could not find attribute value");

    let gecs_update_attribute_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*gecs_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");

    let (_, updated_gecs_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        *gecs_attribute_value.id(),
        Some(*domain_attribute_value.id()),
        gecs_update_attribute_context,
        Some(serde_json::json!["wrongLyrics"]),
        None,
    )
    .await
    .expect("could not update attribute value");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let prefix_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*prefix_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("could not find attribute value");

    let prefix_update_attribute_context =
        AttributeContextBuilder::from(base_attribute_read_context)
            .set_prop_id(*prefix_prop.id())
            .to_context()
            .expect("could not convert builder to attribute context");

    let (_, updated_prefix_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        *prefix_attribute_value.id(),
        Some(*domain_attribute_value.id()),
        prefix_update_attribute_context,
        Some(serde_json::json!["wrong song title"]),
        None,
    )
    .await
    .expect("could not update attribute value");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "hundo_gecs",
                "type": "component",
                "protected": false
            },
            "domain": {
                "thousand_gecs": "wrongLyrics",
                "the_tree_of_clues": "wrong song title",
            }
        }], // actual
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties // expected
    );

    // Ensure that we see the exact expected validation statuses with the exact expected
    // validation errors.
    let validation_statuses = ValidationResolver::find_status(ctx, *component.id())
        .await
        .expect("could not find status for validation(s) of a given component");
    let (match_validation_status, prefix_validation_status) = (
        get_validation_status(&validation_statuses, updated_gecs_attribute_value_id),
        get_validation_status(&validation_statuses, updated_prefix_attribute_value_id),
    );

    // Check match validation errors.
    let mut found_match_validation_error = false;
    for validation_error in &match_validation_status.errors {
        if validation_error.kind == ValidationErrorKind::StringDoesNotEqual {
            if found_match_validation_error {
                panic!("found more than one match validation error: {validation_error:?}");
            }
            found_match_validation_error = true;
        }
    }
    assert!(found_match_validation_error);

    // Check prefix validation errors.
    let mut found_prefix_validation_error = false;
    for validation_error in &prefix_validation_status.errors {
        if validation_error.kind == ValidationErrorKind::StringDoesNotHavePrefix {
            if found_prefix_validation_error {
                panic!("found more than one prefix validation error: {validation_error:?}");
            }
            found_prefix_validation_error = true;
        }
    }
    assert!(found_prefix_validation_error);

    // Update the fields to "valid" values.
    let (_, updated_gecs_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        updated_gecs_attribute_value_id,
        Some(*domain_attribute_value.id()),
        gecs_update_attribute_context,
        Some(serde_json::json!["stupidHorse"]),
        None,
    )
    .await
    .expect("could not update attribute value");
    let (_, updated_prefix_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        updated_prefix_attribute_value_id,
        Some(*domain_attribute_value.id()),
        prefix_update_attribute_context,
        Some(serde_json::json!["toothless"]),
        None,
    )
    .await
    .expect("could not update attribute value");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "hundo_gecs",
                "type": "component",
                "protected": false
            },
            "domain": {
                "thousand_gecs": "stupidHorse",
                "the_tree_of_clues": "toothless"
            }
        }], // actual
        ComponentView::new(ctx, *component.id())
            .await
            .expect("cannot get component view")
            .properties // expected
    );

    // Ensure we see the exact validation status with no errors.
    let validation_statuses = ValidationResolver::find_status(ctx, *component.id())
        .await
        .expect("could not find status for validation(s) of a given component");
    let (match_validation_status, prefix_validation_status) = (
        get_validation_status(&validation_statuses, updated_gecs_attribute_value_id),
        get_validation_status(&validation_statuses, updated_prefix_attribute_value_id),
    );
    assert!(match_validation_status.errors.is_empty());
    assert!(prefix_validation_status.errors.is_empty());
}

#[test]
async fn check_js_validation_for_component(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
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
        FuncBackendKind::JsValidation,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("create js validation func");

    let js_validation_code = "function validate(value) { 
        return { 
            valid: value === 'Temba, his arms open', message: 'Darmok and Jalad at Tanagra'
        };
    }";

    func.set_code_plaintext(ctx, Some(js_validation_code))
        .await
        .expect("set code");
    func.set_handler(ctx, Some("validate"))
        .await
        .expect("set handler");

    let mut builder = ValidationPrototypeContext::builder();
    builder.set_prop_id(*prop.id());
    builder.set_schema_id(*schema.id());
    builder.set_schema_variant_id(*schema_variant.id());
    let validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::json!(null),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("unable to create validation prototype");

    schema_variant
        .finalize(ctx, None)
        .await
        .expect("could not finalize");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let (component, _) = Component::new(ctx, "Danoth", *schema_variant.id())
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

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let properties = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view")
        .properties;

    assert_eq!(
        serde_json::json!({
            "si": {
                "name": "Danoth",
                "type": "component",
                "protected": false
            },
            "domain": {
                "Tamarian": "Shaka, when the walls fell",
            }
        }),
        properties
    );

    let validation_statuses = ValidationResolver::find_status(ctx, *component.id())
        .await
        .expect("could not find status for validation(s) of a given component");

    let status = get_validation_status(&validation_statuses, updated_av_id);

    let darmok: Vec<ValidationError> = vec![ValidationError {
        message: "Darmok and Jalad at Tanagra".to_string(),
        level: None,
        kind: ValidationErrorKind::JsValidation,
        link: None,
    }];

    assert_eq!(darmok, status.errors);

    // Change the function code, re-execute the function and ensure we get back just the
    // latest validation
    let js_validation_code = "function validate(value) { 
        return { 
            valid: value === 'Temba, his arms open', message: 'Darmok and Jalad on the ocean'
        };
    }";

    func.set_code_plaintext(ctx, Some(js_validation_code))
        .await
        .expect("Update validation func code");
    let mut cache: HashMap<PropId, (Option<Value>, AttributeValue)> = HashMap::new();
    component
        .check_single_validation(ctx, &validation_prototype, &mut cache)
        .await
        .expect("check single validation");

    let validation_statuses = ValidationResolver::find_status(ctx, *component.id())
        .await
        .expect("could not find status for validation(s) of a given component");

    let status = get_validation_status(&validation_statuses, updated_av_id);

    let darmok: Vec<ValidationError> = vec![ValidationError {
        message: "Darmok and Jalad on the ocean".to_string(),
        level: None,
        kind: ValidationErrorKind::JsValidation,
        link: None,
    }];
    assert_eq!(darmok, status.errors);

    let av = AttributeValue::get_by_id(ctx, &updated_av_id)
        .await
        .expect("get updated av")
        .expect("not none");
    let pav = av
        .parent_attribute_value(ctx)
        .await
        .expect("could not get parent attribute value");

    let (_, updated_av_id) = AttributeValue::update_for_context(
        ctx,
        *av.id(),
        pav.map(|pav| *pav.id()),
        av_update_context,
        Some(serde_json::json!("Temba, his arms open")),
        None,
    )
    .await
    .expect("update attr value");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let properties = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view")
        .properties;

    assert_eq!(
        serde_json::json!({
            "si": {
                "name": "Danoth",
                "type": "component",
                "protected": false
            },
            "domain": {
                "Tamarian": "Temba, his arms open",
            }
        }),
        properties
    );

    let validation_statuses = ValidationResolver::find_status(ctx, *component.id())
        .await
        .expect("could not find status for validation(s) of a given component");

    let status = get_validation_status(&validation_statuses, updated_av_id);

    let empty: Vec<ValidationError> = vec![];
    assert_eq!(empty, status.errors);
}

fn get_validation_status(
    validation_statuses: &[ValidationStatus],
    attribute_value_id: AttributeValueId,
) -> ValidationStatus {
    let mut the_validation_status = None;
    for validation_status in validation_statuses {
        if validation_status.attribute_value_id == attribute_value_id {
            if the_validation_status.is_some() {
                panic!(
                    "found more than one validation status for that attribute_value_id: {validation_statuses:?}"
                );
            }
            the_validation_status = Some(validation_status.clone());
        }
    }
    the_validation_status.expect("did not find a validation status")
}

/// This test ensures that validation statuses correspond to attribute values that exist in an
/// attribute context that we expect (schema, schema variant, and component).
#[test]
async fn ensure_validations_are_sourced_correctly(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let component_payload = harness
        .create_component(ctx, "ksg", Builtin::AwsRegion)
        .await;

    let updated_region_attribute_value_id = component_payload
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/region",
            Some(serde_json::json!["us-east-1"]),
        )
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "us-east-1",
                "color": "#FF9900",
                "type": "configurationFrame",
                "protected": false
            },

            "domain": {
                "region": "us-east-1",
            }
        }], // actual
        component_payload
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // expected
    );

    // Ensure that we see exactly one expected validation status with exactly one expected
    // validation error.
    let validation_statuses = ValidationResolver::find_status(ctx, component_payload.component_id)
        .await
        .expect("could not find status for validation(s) of a given component");

    let mut expected_validation_status = None;
    for validation_status in &validation_statuses {
        // Ensure that the attribute values found are of relevant attribute contexts.
        let attribute_value = AttributeValue::get_by_id(ctx, &validation_status.attribute_value_id)
            .await
            .expect("could not get attribute value by id")
            .expect("attribute value not found by id");
        assert_eq!(
            attribute_value.context.component_id(),
            component_payload.component_id
        );

        // Now, we can find the expected validation status.
        if validation_status.attribute_value_id == updated_region_attribute_value_id {
            if expected_validation_status.is_some() {
                panic!("found more than one expected validation status: {validation_statuses:?}");
            }
            expected_validation_status = Some(validation_status.clone());
        }
    }
    let expected_validation_status =
        expected_validation_status.expect("did not find expected validation status");
    assert!(expected_validation_status.errors.is_empty());
}
