use crate::dal::test;
use dal::socket::input::InputSocket;
use dal::test_harness::{
    create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant_with_root,
};
use dal::{AttributeReadContext, DalContext, PropKind, SchemaKind, StandardModel};

use pretty_assertions_sorted::assert_eq;

#[test]
async fn list_for_schema_variant(ctx: &DalContext<'_, '_>) {
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

    let _input_socket = InputSocket::new(
        ctx,
        *value_prop.id(),
        *schema.id(),
        *schema_variant.id(),
        Some("value".to_string()),
        true,
        None,
    )
    .await
    .expect("could not create input socket");

    let mut found_input_sockets = InputSocket::list_for_schema_variant(ctx, *schema_variant.id())
        .await
        .expect("could not get input sockets for schema variant id");
    let found_input_socket = found_input_sockets
        .pop()
        .expect("found input sockets are empty");
    assert_eq!(found_input_sockets.len(), 0);
    assert_eq!(found_input_socket.prop_id(), value_prop.id());
    assert_eq!(found_input_socket.schema_id(), schema.id());
    assert_eq!(found_input_socket.schema_variant_id(), schema_variant.id());
}
