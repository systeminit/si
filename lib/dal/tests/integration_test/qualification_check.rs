use dal::{
    test_harness::{create_qualification_check, create_schema, create_schema_variant},
    HistoryActor, QualificationCheck, SchemaKind, StandardModel, Tenancy, Visibility,
};
use test_env_log::test;

use crate::test_setup;

#[test(tokio::test)]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let qualification_check = QualificationCheck::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "checkit",
    )
    .await
    .expect("cannot create qualification check");
    assert_eq!(qualification_check.name(), "checkit");
}

#[test(tokio::test)]
async fn schema_variants() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let qualification_check =
        create_qualification_check(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let variant = create_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech,
        encr_key,
    )
    .await;
    variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate schema variant with schema");

    qualification_check
        .add_schema_variant(&txn, &nats, &visibility, &history_actor, variant.id())
        .await
        .expect("cannot add schema variant to qualification check");
    let variants = qualification_check
        .schema_variants(&txn, &visibility)
        .await
        .expect("cannot get schema variants");
    assert_eq!(variants, vec![variant.clone()]);

    qualification_check
        .remove_schema_variant(&txn, &nats, &visibility, &history_actor, variant.id())
        .await
        .expect("cannot remove schema variant");
    let variants = qualification_check
        .schema_variants(&txn, &visibility)
        .await
        .expect("cannot get schema variants");
    assert_eq!(variants, vec![]);
}
