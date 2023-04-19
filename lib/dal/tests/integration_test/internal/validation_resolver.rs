use dal::{
    func::{backend::validation::FuncBackendValidationArgs, binding::FuncBinding},
    validation::Validation,
    AttributeContext, AttributeValue, DalContext, Func, FuncBackendKind, FuncBackendResponseType,
    Prop, PropKind, StandardModel, ValidationPrototype, ValidationPrototypeContext,
    ValidationResolver,
};
use dal_test::{
    test,
    test_harness::{create_component_for_schema, create_schema, create_schema_variant_with_root},
};

#[test]
async fn new(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    let prop = Prop::new(
        ctx,
        "glaive",
        PropKind::String,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
    )
    .await
    .expect("could not create prop");

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

    schema_variant
        .finalize(ctx, None)
        .await
        .expect("could not finalize schema variant");
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
