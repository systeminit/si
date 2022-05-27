use dal::DalContext;

use crate::dal::test;
use dal::{
    func::{backend::validation::FuncBackendValidateStringValueArgs, binding::FuncBinding},
    test_harness::create_component_for_schema,
    validation_prototype::ValidationPrototypeContext,
    AttributeContext, AttributeValue, Func, FuncBackendKind, FuncBackendResponseType, Schema,
    StandardModel, SystemId, ValidationPrototype, ValidationResolver,
};

const UNSET_ID_VALUE: i64 = -1;

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let default_variant = schema
        .default_variant(ctx)
        .await
        .expect("cannot find default variant");

    let prop = default_variant
        .props(ctx)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let func = Func::new(
        ctx,
        "test:validateString",
        FuncBackendKind::ValidateStringValue,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let mut context = ValidationPrototypeContext::new();
    context.set_prop_id(*prop.id());
    context.set_schema_id(*schema.id());
    context.set_schema_variant_id(*default_variant.id());
    let prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringValueArgs::new(
            None,
            "amon amarth".to_owned(),
        ))
        .expect("cannot turn args into json"),
        context.clone(),
    )
    .await
    .expect("unable to create validation prototype");

    let component = create_component_for_schema(ctx, schema.id()).await;

    let args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    let func_binding_return_value = func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    // Note: This is kinda wrong, the func_binding_return_value (and the func_binding) will point to the validation execution
    // But we want the actual inner value that was used in the validation
    // Since we never bothered to generate one we just use the validation as a substitute that properly tests the code, but doesn't make sense in the product
    let context = AttributeContext::builder()
        .set_prop_id(*prop.id())
        .set_component_id(*component.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*default_variant.id())
        .to_context()
        .expect("unable to build attribute context");
    let attribute_value = AttributeValue::new(
        ctx,
        *func_binding.id(),
        *func_binding_return_value.id(),
        context,
        Option::<&str>::None,
    )
    .await
    .expect("unable to create attribute value");

    let _validation_resolver = ValidationResolver::new(
        ctx,
        *prototype.id(),
        *attribute_value.id(),
        *func_binding.id(),
    )
    .await
    .expect("cannot create new attribute resolver");
}

#[test]
async fn find_errors(ctx: &DalContext<'_, '_>) {
    let unset_system_id: SystemId = UNSET_ID_VALUE.into();

    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let default_variant = schema
        .default_variant(ctx)
        .await
        .expect("cannot find default variant");

    let prop = default_variant
        .props(ctx)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let func = Func::new(
        ctx,
        "test:validateString",
        FuncBackendKind::ValidateStringValue,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let mut context = ValidationPrototypeContext::new();
    context.set_prop_id(*prop.id());
    context.set_schema_id(*schema.id());
    context.set_schema_variant_id(*default_variant.id());
    let first_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringValueArgs::new(
            None,
            "amon amarth".to_owned(),
        ))
        .expect("cannot turn args into json"),
        context.clone(),
    )
    .await
    .expect("unable to create validation prototype");

    let second_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidateStringValueArgs::new(
            None,
            "twisty monkey".to_owned(),
        ))
        .expect("cannot turn args into json"),
        context,
    )
    .await
    .expect("unable to create validation prototype");

    let first_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());
    let first_func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(first_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");

    let first_func_binding_return_value = first_func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let component = create_component_for_schema(ctx, schema.id()).await;

    // Note: This is kinda wrong, the func_binding_return_value (and the func_binding) will point to the validation execution
    // But we want the actual inner value that was used in the validation
    // Since we never bothered to generate one we just use the validation as a substitute that properly tests the code, but doesn't make sense in the product
    let context = AttributeContext::builder()
        .set_prop_id(*prop.id())
        .set_component_id(*component.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*default_variant.id())
        .to_context()
        .expect("unable to build attribute context");
    let attribute_value = AttributeValue::new(
        ctx,
        *first_func_binding.id(),
        *first_func_binding_return_value.id(),
        context,
        Option::<&str>::None,
    )
    .await
    .expect("unable to create attribute value");

    let _first_validation_resolver = ValidationResolver::new(
        ctx,
        *first_prototype.id(),
        *attribute_value.id(),
        *first_func_binding.id(),
    )
    .await
    .expect("cannot create new validation resolver");

    let second_args = FuncBackendValidateStringValueArgs::new(
        Some("not twisty monkey".to_string()),
        "twisty monkey".to_string(),
    );
    let second_func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(second_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");

    let _second_func_binding_return_value = second_func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let _second_validation_resolver = ValidationResolver::new(
        ctx,
        *second_prototype.id(),
        *attribute_value.id(),
        *second_func_binding.id(),
    )
    .await
    .expect("cannot create new validation resolver");

    let mut validation_results =
        ValidationResolver::find_status(ctx, *component.id(), unset_system_id)
            .await
            .expect("cannot find values");
    // Order of output from find_status above isn't stable. Order the
    // results by the AttributeValueId that they are for, so we have
    // something stable to compare against in the asserts below.
    validation_results
        .sort_by(|a, b| i64::from(a.attribute_value_id).cmp(&i64::from(b.attribute_value_id)));

    // There are two results, because the first one is showing that the
    // "directly on the prop" AttributeValue does not have any
    // validation errors.
    assert_eq!(2, validation_results.len());
    assert_eq!(0, validation_results[0].errors.len());
    assert_eq!(2, validation_results[1].errors.len());
    assert_eq!(
        "value () does not match expected (amon amarth)",
        validation_results[1].errors[0].message,
    );
    assert_eq!(
        "value (not twisty monkey) does not match expected (twisty monkey)",
        validation_results[1].errors[1].message,
    );
}
