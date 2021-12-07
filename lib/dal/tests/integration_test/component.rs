use crate::test_setup;

use dal::test_harness::{create_schema, create_schema_variant};
use dal::{Component, HistoryActor, SchemaKind, StandardModel, Tenancy, Visibility};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create entity");
}

#[tokio::test]
async fn schema_relationships() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Implementation,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant to schema");
    let component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create entity");
    component
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema for entity");
    component
        .set_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot set schema for entity");
}
