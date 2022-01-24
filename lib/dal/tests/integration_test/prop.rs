use dal::{
    test_harness::{create_prop, create_schema_variant},
    HistoryActor, Prop, PropKind, StandardModel, Tenancy, Visibility,
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
        _veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let prop = Prop::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coolness",
        PropKind::String,
    )
    .await
    .expect("cannot create prop");
    assert_eq!(prop.name(), "coolness");
    assert_eq!(prop.kind(), &PropKind::String);
}

#[tokio::test]
async fn schema_variants() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let prop = create_prop(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    prop.add_schema_variant(
        &txn,
        &nats,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await
    .expect("cannot add schema variant");

    let relations = prop
        .schema_variants(&txn, &visibility)
        .await
        .expect("cannot get schema variants");
    assert_eq!(relations, vec![schema_variant.clone()]);

    prop.remove_schema_variant(
        &txn,
        &nats,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await
    .expect("cannot remove schema variant");

    let relations = prop
        .schema_variants(&txn, &visibility)
        .await
        .expect("cannot get schema variants");
    assert_eq!(relations, vec![]);
}
