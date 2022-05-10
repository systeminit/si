use crate::dal::test;
use dal::DalContext;
use dal::{schema::SchemaVariant, test_harness::create_schema, SchemaKind, StandardModel};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Concrete).await;

    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "ringo")
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(variant.name(), "ringo");
}

#[test]
async fn set_schema(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema ui menu");

    let attached_schema = variant
        .schema(ctx)
        .await
        .expect("cannot get schema")
        .expect("should have a schema");
    assert_eq!(schema, attached_schema);

    variant
        .unset_schema(ctx)
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = variant.schema(ctx).await.expect("cannot get schema");
    assert_eq!(attached_schema, None);
}
