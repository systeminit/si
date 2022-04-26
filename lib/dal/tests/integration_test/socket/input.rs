use crate::dal::test;
use dal::socket::input::InputSocket;

use dal::test_harness::{create_schema, create_schema_variant_with_root};

use dal::{DalContext, SchemaKind, StandardModel};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn list_for_schema_variant(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, _root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema_variant
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let input_sockets = InputSocket::list_for_schema_variant(ctx, *schema_variant.id())
        .await
        .expect("could not get input sockets for schema variant id");

    assert_eq!(input_sockets, vec![]);
}
