use crate::dal::test;
use dal::{
    attribute::context::AttributeContext,
    attribute::prototype::AttributePrototype,
    func::{backend::string::FuncBackendStringArgs, binding::FuncBinding},
    test_harness::{create_schema, create_schema_variant_with_root},
    AttributeReadContext, Func, FuncBackendKind, FuncBackendResponseType, PropKind, SchemaKind,
    StandardModel,
};
use dal::{AttributePrototypeArgument, DalContext, InternalProvider};

use dal::test_harness::create_prop_of_kind_and_set_parent_with_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_and_list_for_attribute_prototype(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema_variant
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    // domain: Object
    // └─ object: Object
    //    └─ name: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root_prop.domain_prop_id,
        base_attribute_read_context,
    )
    .await;
    let name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;

    let func = Func::new(
        ctx,
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");
    let args = FuncBackendStringArgs::new("starfield".to_string());
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
    let context = AttributeContext::builder()
        .set_prop_id(*name_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .to_context()
        .expect("cannot create context");

    let attribute_prototype = AttributePrototype::new(
        ctx,
        *func.id(),
        *func_binding.id(),
        *func_binding_return_value.id(),
        context,
        None,
        None,
        None,
    )
    .await
    .expect("cannot create new attribute prototype");

    let internal_provider = InternalProvider::new(
        ctx,
        *name_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        Some("name".to_string()),
        true,
        None,
        None,
    )
    .await
    .expect("could not create internal provider");

    let argument = AttributePrototypeArgument::new(
        ctx,
        "title".to_string(),
        internal_provider.id(),
        attribute_prototype.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    let mut found_arguments =
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *attribute_prototype.id())
            .await
            .expect("could not list attribute prototype argument for attribute prototype");
    let found_argument = found_arguments
        .pop()
        .expect("found attribute prototype arguments are empty");
    if !found_arguments.is_empty() {
        panic!("expected empty: found attribute prototype arguments returned more results than expected");
    }

    assert_eq!(found_argument.name(), argument.name());
    assert_eq!(
        found_argument.internal_provider_id(),
        argument.internal_provider_id()
    );
    let found_prototype = found_argument
        .attribute_prototype(ctx)
        .await
        .expect("could not get attribute prototype for attribute prototype argument")
        .expect("attribute prototype for attribute prototype argument not found");
    assert_eq!(found_prototype.id(), attribute_prototype.id());
}
