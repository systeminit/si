use crate::dal::test;
use dal::attribute::context::AttributeContextBuilder;
use dal::func::backend::string::FuncBackendStringArgs;
use dal::func::binding::FuncBinding;
use dal::provider::external::ExternalProvider;
use dal::provider::internal::InternalProvider;
use dal::test_harness::{
    create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant_with_root,
};
use dal::AttributePrototype;
use dal::{
    AttributeReadContext, DalContext, Func, FuncBackendKind, FuncBackendResponseType, PropKind,
    SchemaKind, StandardModel,
};

use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_and_list_for_schema_variant(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
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
    //    ├─ name: String
    //    └─ value: String
    let object_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::Object,
        "object",
        root_prop.domain_prop_id,
        base_attribute_read_context,
    )
    .await;
    let _name_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "name",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;
    let value_prop = create_prop_of_kind_and_set_parent_with_name(
        ctx,
        PropKind::String,
        "value",
        *object_prop.id(),
        base_attribute_read_context,
    )
    .await;

    let _internal_provider = InternalProvider::new(
        ctx,
        *value_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        Some("value".to_string()),
        true,
        None,
        None,
    )
    .await
    .expect("could not create internal provider");

    let mut found_internal_providers =
        InternalProvider::list_for_schema_variant(ctx, *schema_variant.id())
            .await
            .expect("could not get internal providers for schema variant id");
    let found_internal_provider = found_internal_providers
        .pop()
        .expect("found internal providers are empty");
    assert_eq!(found_internal_providers.len(), 0);
    assert_eq!(found_internal_provider.prop_id(), value_prop.id());
    assert_eq!(found_internal_provider.schema_id(), schema.id());
    assert_eq!(
        found_internal_provider.schema_variant_id(),
        schema_variant.id()
    );
    assert_eq!(*found_internal_provider.internal_consumer(), true);

    // FIXME(nick): this ExternalProvider setup is straight up wrong. For now, we just want to make
    // sure that the "create and list" workflow works. The attribute prototype created is purely
    // dummy data.
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
        serde_json::to_value(args).expect("cannot convert args into serde_json::Value"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
    let func_binding_return_value = func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");
    let attribute_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(*value_prop.id())
        .to_context()
        .expect("cannot create attribute context");
    let attribute_prototype = AttributePrototype::new(
        ctx,
        *func.id(),
        *func_binding.id(),
        *func_binding_return_value.id(),
        attribute_context,
        None,
        None,
        None,
    )
    .await
    .expect("cannot create new attribute prototype");
    let _external_provider = ExternalProvider::new(
        ctx,
        *value_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        Some("value".to_string()),
        None,
        *attribute_prototype.id(),
    )
    .await
    .expect("could not create external provider");

    let mut found_external_providers =
        ExternalProvider::list_for_schema_variant(ctx, *schema_variant.id())
            .await
            .expect("could not get external providers for schema variant id");
    let found_external_provider = found_external_providers
        .pop()
        .expect("found external providers are empty");
    assert_eq!(found_external_providers.len(), 0);
    assert_eq!(found_external_provider.prop_id(), value_prop.id());
    assert_eq!(found_external_provider.schema_id(), schema.id());
    assert_eq!(
        found_external_provider.schema_variant_id(),
        schema_variant.id()
    );
    assert_eq!(
        found_external_provider.attribute_prototype_id(),
        attribute_prototype.id()
    );
}
