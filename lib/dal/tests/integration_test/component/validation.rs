use pretty_assertions_sorted::assert_eq;

use dal::func::backend::validation::ValidationKind;

use dal::attribute::context::AttributeContextBuilder;
use dal::func::backend::validation::validate_string::FuncBackendValidateStringValueArgs;

use dal::test_harness::{
    create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root,
};
use dal::validation_prototype::ValidationPrototypeContext;
use dal::{
    AttributeReadContext, AttributeValue, Component, ComponentView, DalContext, Func, PropKind,
    SchemaKind, StandardModel, SystemId, ValidationPrototype, ValidationResolver,
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

    let mut validation_context = ValidationPrototypeContext::new();
    validation_context.set_prop_id(*gecs_prop.id());
    validation_context.set_schema_id(*schema.id());
    validation_context.set_schema_variant_id(*schema_variant.id());

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("could not perform find by attr");
    let func = funcs.pop().expect("could not find func");

    let expected = "stupidHorse".to_string();
    let args = serde_json::to_value(FuncBackendValidateStringValueArgs::new(
        None,
        expected.clone(),
    ))
    .expect("could not convert args to Value");
    ValidationPrototype::new(ctx, *func.id(), args, validation_context)
        .await
        .expect("could not create validation prototype");

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

    let update_attribute_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*gecs_prop.id())
        .to_context()
        .expect("could not convert builder to attribute context");

    let (_, updated_gecs_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        *gecs_attribute_value.id(),
        Some(*domain_attribute_value.id()),
        update_attribute_context,
        Some(serde_json::json!["wrongLyrics"]),
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
                "thousand_gecs": "wrongLyrics"
            }
        }], // actual
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // expected
    );

    // Ensure that we see exactly one expected validation status with exactly one expected
    // validation error.
    let validation_statuses = ValidationResolver::find_status(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find status for validation(s) of a given component");

    let mut expected_validation_status = None;
    for validation_status in &validation_statuses {
        if validation_status.attribute_value_id == updated_gecs_attribute_value_id {
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

    let mut found_expected_validation_error = false;
    for validation_error in &expected_validation_status.errors {
        if validation_error.kind == ValidationKind::ValidateString {
            if found_expected_validation_error {
                panic!(
                    "found more than one expected validation error: {:?}",
                    validation_error
                );
            }
            found_expected_validation_error = true;
        }
    }
    assert!(found_expected_validation_error);

    // Update the target field to a "valid" value.
    let (_, updated_gecs_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        updated_gecs_attribute_value_id,
        Some(*domain_attribute_value.id()),
        update_attribute_context,
        Some(serde_json::json![expected]),
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
                "thousand_gecs": "stupidHorse"
            }
        }], // actual
        ComponentView::for_context(ctx, base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties // expected
    );

    // Ensure we see exactly one expected validation status with no errors.
    let validation_statuses = ValidationResolver::find_status(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find status for validation(s) of a given component");

    let mut expected_validation_status = None;
    for validation_status in &validation_statuses {
        if validation_status.attribute_value_id == updated_gecs_attribute_value_id {
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
