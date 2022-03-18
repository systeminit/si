use dal::{
    component::ComponentKind, schema::SchemaVariant, test_harness::create_schema, HistoryActor,
    Schema, SchemaKind, StandardModel, Visibility, WriteTenancy,
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
        veritech,
        encr_key,
    );
    let write_tenancy = WriteTenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &(&write_tenancy).into(),
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;

    let (variant, _) = SchemaVariant::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        *schema.id(),
        "ringo",
        veritech,
        encr_key,
    )
    .await
    .expect("cannot create schema ui menu");
    assert_eq!(variant.name(), "ringo");
}

#[test(tokio::test)]
async fn set_schema() {
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
    let write_tenancy = WriteTenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &(&write_tenancy).into(),
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let (variant, _) = SchemaVariant::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        *schema.id(),
        "v0",
        veritech,
        encr_key,
    )
    .await
    .expect("cannot create schema ui menu");

    variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = variant
        .schema(&txn, &visibility)
        .await
        .expect("cannot get schema")
        .expect("should have a schema");
    assert_eq!(schema, attached_schema);

    variant
        .unset_schema(&txn, &nats, &visibility, &history_actor)
        .await
        .expect("cannot associate ui menu with schema");
    let attached_schema = variant
        .schema(&txn, &visibility)
        .await
        .expect("cannot get schema");
    assert_eq!(attached_schema, None);
}
