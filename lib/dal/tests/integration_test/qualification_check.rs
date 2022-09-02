use crate::dal::test;
use dal::DalContext;
use dal::{
    test_harness::{create_qualification_check, create_schema, create_schema_variant},
    HistoryActor, QualificationCheck, SchemaKind, StandardModel, Visibility, WriteTenancy,
};

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let qualification_check = QualificationCheck::new(ctx, "checkit")
        .await
        .expect("cannot create qualification check");
    assert_eq!(qualification_check.name(), "checkit");
}

#[test]
async fn schema_variants(ctx: &DalContext<'_, '_, '_>) {
    let qualification_check = create_qualification_check(ctx).await;
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let variant = create_schema_variant(ctx, *schema.id()).await;

    qualification_check
        .add_schema_variant(ctx, variant.id())
        .await
        .expect("cannot add schema variant to qualification check");
    let variants = qualification_check
        .schema_variants(ctx)
        .await
        .expect("cannot get schema variants");
    assert_eq!(variants, vec![variant.clone()]);

    qualification_check
        .remove_schema_variant(ctx, variant.id())
        .await
        .expect("cannot remove schema variant");
    let variants = qualification_check
        .schema_variants(ctx)
        .await
        .expect("cannot get schema variants");
    assert_eq!(variants, vec![]);
}
