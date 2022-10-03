use pretty_assertions_sorted::assert_eq;

use dal::attribute::context::AttributeContextBuilder;
use dal::func::backend::validation::FuncBackendValidationArgs;
use dal::test::helpers::builtins::{Builtin, SchemaBuiltinsTestHarness};
use dal::test_harness::{
    create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root,
};
use dal::validation::{Validation, ValidationErrorKind};
use dal::ValidationPrototypeContext;
use dal::ValidationStatus;
use dal::{
    AttributeReadContext, AttributeValue, AttributeValueId, Component, ComponentView, DalContext,
    Func, PropKind, SchemaKind, StandardModel, SystemId, ValidationPrototype, ValidationResolver,
};

use crate::dal::test;

#[test]
async fn check_validations_for_component(ctx: &DalContext) {
    // Setup the schema and schema variant and create a validation for a string field.
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let gecs_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "thousand_gecs").await;
    gecs_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await
        .expect("cannot set parent prop");
    let prefix_prop =
        create_prop_of_kind_with_name(ctx, PropKind::String, "the_tree_of_clues").await;
    prefix_prop
        .set_parent_prop(ctx, root_prop.domain_prop_id)
        .await
        .expect("cannot set parent prop");

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
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    // Once setup is complete, create a component and update the target field to an "invalid" value.
    let (component, _) =
        Component::new_for_schema_variant_with_node(ctx, "hundo_gecs", schema_variant.id())
            .await
            .expect("could not create component");

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
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

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "hundo_gecs"
            },
            "domain": {
                "the_tree_of_clues": "wrong song title",
                "thousand_gecs": "wrongLyrics"
            }
        }], // actual
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // expected
    );

    // Ensure that we see the exact expected validation statuses with the exact expected
    // validation errors.
    let validation_statuses = ValidationResolver::find_status(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find status for validation(s) of a given component");
    let (match_validation_status, prefix_validation_status) = get_validation_statuses(
        &validation_statuses,
        updated_gecs_attribute_value_id,
        updated_prefix_attribute_value_id,
    );

    // Check match validation errors.
    let mut found_match_validation_error = false;
    for validation_error in &match_validation_status.errors {
        if validation_error.kind == ValidationErrorKind::StringDoesNotEqual {
            if found_match_validation_error {
                panic!(
                    "found more than one match validation error: {:?}",
                    validation_error
                );
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
                panic!(
                    "found more than one prefix validation error: {:?}",
                    validation_error
                );
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

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "hundo_gecs"
            },
            "domain": {
                "the_tree_of_clues": "toothless",
                "thousand_gecs": "stupidHorse"
            }
        }], // actual
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // expected
    );

    // Ensure we see the exact validation status with no errors.
    let validation_statuses = ValidationResolver::find_status(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find status for validation(s) of a given component");
    let (match_validation_status, prefix_validation_status) = get_validation_statuses(
        &validation_statuses,
        updated_gecs_attribute_value_id,
        updated_prefix_attribute_value_id,
    );
    assert!(match_validation_status.errors.is_empty());
    assert!(prefix_validation_status.errors.is_empty());
}

fn get_validation_statuses(
    validation_statuses: &Vec<ValidationStatus>,
    match_attribute_value_id: AttributeValueId,
    prefix_attribute_value_id: AttributeValueId,
) -> (ValidationStatus, ValidationStatus) {
    let mut match_validation_status = None;
    let mut prefix_validation_status = None;
    for validation_status in validation_statuses {
        if validation_status.attribute_value_id == match_attribute_value_id {
            if match_validation_status.is_some() {
                panic!(
                    "found more than one match validation status: {:?}",
                    validation_statuses
                );
            }
            match_validation_status = Some(validation_status.clone());
        } else if validation_status.attribute_value_id == prefix_attribute_value_id {
            if prefix_validation_status.is_some() {
                panic!(
                    "found more than one prefix validation status: {:?}",
                    validation_statuses
                );
            }
            prefix_validation_status = Some(validation_status.clone());
        }
    }
    (
        match_validation_status.expect("did not find match validation status"),
        prefix_validation_status.expect("did not find prefix validation status"),
    )
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

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ksg"
            },
            "domain": {
                "region": "us-east-1"
            }
        }], // actual
        component_payload.component_view_properties(ctx).await // expected
    );

    // Ensure that we see exactly one expected validation status with exactly one expected
    // validation error.
    let validation_statuses =
        ValidationResolver::find_status(ctx, component_payload.component_id, SystemId::NONE)
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
            attribute_value.context.schema_id(),
            component_payload.schema_id
        );
        assert_eq!(
            attribute_value.context.schema_variant_id(),
            component_payload.schema_variant_id
        );
        assert_eq!(
            attribute_value.context.component_id(),
            component_payload.component_id
        );

        // Now, we can find the expected validation status.
        if validation_status.attribute_value_id == updated_region_attribute_value_id {
            if expected_validation_status.is_some() {
                panic!(
                    "found more than one expected validation status: {:?}",
                    validation_statuses
                );
            }
            expected_validation_status = Some(validation_status.clone());
        }
    }
    let expected_validation_status =
        expected_validation_status.expect("did not find expected validation status");
    assert!(expected_validation_status.errors.is_empty());
}
