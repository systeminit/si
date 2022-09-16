use crate::dal::test;
use dal::{
    attribute::context::AttributeContext,
    attribute::prototype::AttributePrototype,
    func::{backend::string::FuncBackendStringArgs, binding::FuncBinding},
    test_harness::{create_schema, create_schema_variant_with_root},
    Func, FuncBackendKind, FuncBackendResponseType, PropKind, SchemaKind, StandardModel,
};
use dal::{AttributePrototypeArgument, DalContext, InternalProvider};

use dal::test_harness::create_prop_of_kind_and_set_parent_with_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_and_list_for_attribute_prototype(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    // domain: Object
    // └─ object: Object
    //    └─ name: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root_prop.domain_prop_id,
    )
    .await;
    let name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
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
    )
    .await
    .expect("cannot create new attribute prototype");

    let internal_provider =
        InternalProvider::new_implicit(ctx, *name_prop.id(), *schema.id(), *schema_variant.id())
            .await
            .expect("could not create internal provider");

    let argument = AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *attribute_prototype.id(),
        "title",
        *internal_provider.id(),
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
    assert_eq!(
        found_argument.attribute_prototype_id(),
        *attribute_prototype.id()
    );
}
