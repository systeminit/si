use dal::{
    func::{backend::validation::FuncBackendValidationArgs, binding::FuncBinding},
    validation::Validation,
    AttributeContext, AttributeValue, DalContext, Func, FuncBackendKind, FuncBackendResponseType,
    PropKind, StandardModel, ValidationPrototype, ValidationPrototypeContext, ValidationResolver,
};
use dal_test::test_harness::create_prop_and_set_parent;
use dal_test::{
    test,
    test_harness::{create_component_for_schema, create_schema, create_schema_variant_with_root},
};

#[test]
async fn new(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let prop =
        create_prop_and_set_parent(ctx, PropKind::String, "glaive", root_prop.domain_prop_id).await;

    let func = Func::new(
        ctx,
        "test:validateString",
        FuncBackendKind::Validation,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let mut builder = ValidationPrototypeContext::builder();
    builder.set_prop_id(*prop.id());
    builder.set_schema_id(*schema.id());
    builder.set_schema_variant_id(*schema_variant.id());
    let prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidationArgs::new(Validation::StringEquals {
            value: None,
            expected: "amon amarth".to_owned(),
        }))
        .expect("cannot turn args into json"),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("unable to create validation prototype");

    let component = create_component_for_schema(ctx, schema.id()).await;

    let args = FuncBackendValidationArgs::new(Validation::StringEquals {
        value: Some("".to_string()),
        expected: "amon amarth".to_string(),
    });
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
async fn find_errors(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let prop =
        create_prop_and_set_parent(ctx, PropKind::String, "glaive", root_prop.domain_prop_id).await;

    let func = Func::new(
        ctx,
        "test:validateString",
        FuncBackendKind::Validation,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let mut builder = ValidationPrototypeContext::builder();
    builder.set_prop_id(*prop.id());
    builder.set_schema_id(*schema.id());
    builder.set_schema_variant_id(*schema_variant.id());
    let first_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidationArgs::new(Validation::StringEquals {
            value: None,
            expected: "amon amarth".to_owned(),
        }))
        .expect("cannot turn args into json"),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("unable to create validation prototype");

    let second_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(FuncBackendValidationArgs::new(Validation::StringEquals {
            value: None,
            expected: "twisty monkey".to_owned(),
        }))
        .expect("cannot turn args into json"),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("unable to create validation prototype");

    let first_args = FuncBackendValidationArgs::new(Validation::StringEquals {
        value: Some("".to_string()),
        expected: "amon amarth".to_string(),
    });
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

    let second_args = FuncBackendValidationArgs::new(Validation::StringEquals {
        value: Some("not twisty monkey".to_string()),
        expected: "twisty monkey".to_string(),
    });
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

    let mut validation_results = ValidationResolver::find_status(ctx, *component.id())
        .await
        .expect("cannot find values");

    let mut got_results = false;
    for result in &mut validation_results {
        let av = AttributeValue::get_by_id(ctx, &result.attribute_value_id)
            .await
            .unwrap()
            .unwrap();
        if av.context.prop_id() == *prop.id() {
            assert_eq!(2, result.errors.len());
            // Order of the individual error messages isn't stable, so we'll sort them lexicographically.
            result.errors.sort_by(|a, b| a.message.cmp(&b.message));
            assert_eq!(
                "value () does not match expected (amon amarth)",
                &result.errors[0].message,
            );
            assert_eq!(
                "value (not twisty monkey) does not match expected (twisty monkey)",
                &result.errors[1].message,
            );
            got_results = true;
        } else {
            assert_eq!(0, result.errors.len());
        }
    }
    assert!(got_results, "got expected results");
}
