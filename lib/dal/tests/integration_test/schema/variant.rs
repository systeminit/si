use dal::{
    schema::SchemaVariant, test_harness::create_schema, HistoryActor, SchemaKind, StandardModel,
    Tenancy, Visibility,
};

use crate::test_setup;

#[tokio::test]
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
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let (variant, _) = SchemaVariant::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "ringo",
        veritech,
        encr_key,
    )
    .await
    .expect("cannot create schema ui menu");
    assert_eq!(variant.name(), "ringo");
}

#[tokio::test]
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
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let (variant, _) = SchemaVariant::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
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
