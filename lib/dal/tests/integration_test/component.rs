use crate::test_setup;

use dal::test_harness::{create_schema, create_schema_variant};
use dal::{
    Component, ComponentQualificationView, HistoryActor, Prop, PropKind, SchemaKind, StandardModel,
    Tenancy, Visibility,
};
use serde_json::json;

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
        _veritech,
    );
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
async fn new_for_schema_variant_with_node() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
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
        &SchemaKind::Concept,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant");

    let _component = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        schema_variant.id(),
    )
    .await
    .expect("cannot create component");
}

#[tokio::test]
async fn schema_relationships() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
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

#[tokio::test]
async fn qualification_view() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
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
    let prop = Prop::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "some_property",
        PropKind::String,
    )
    .await
    .expect("cannot create prop");
    prop.add_schema_variant(
        &txn,
        &nats,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await
    .expect("cannot add schema variant for prop");

    let component_qualification_view =
        ComponentQualificationView::new(&txn, &tenancy, &visibility, component.id())
            .await
            .expect("cannot create ComponentQualificationView");

    assert_eq!(
        serde_json::to_value(&component_qualification_view)
            .expect("cannot serialize ComponentQualificationView"),
        json!({
            "name": "mastodon",
            "properties": {
                "some_property": null,
                "name": "mastodon"
            }
        }),
    );
}
