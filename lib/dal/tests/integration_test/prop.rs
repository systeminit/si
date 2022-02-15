use dal::{
    test_harness::{create_prop, create_prop_of_kind, create_schema_variant},
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
        veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let prop = Prop::new(
        &txn,
        &nats,
        veritech,
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
        veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let prop = create_prop(&txn, &nats, veritech, &tenancy, &visibility, &history_actor).await;

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

#[tokio::test]
async fn parent_props() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let parent_prop = create_prop_of_kind(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
    )
    .await;
    let child_prop = create_prop_of_kind(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
    )
    .await;

    child_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *parent_prop.id())
        .await
        .expect("cannot set parent prop");
    let retrieved_parent_prop = child_prop
        .parent_prop(&txn, &visibility)
        .await
        .expect("cannot get parent prop")
        .expect("there was no parent prop and we expected one!");
    assert_eq!(retrieved_parent_prop, parent_prop);

    let children = parent_prop
        .child_props(&txn, &tenancy, &visibility)
        .await
        .expect("should have children");
    assert_eq!(children, vec![child_prop]);
}

#[tokio::test]
async fn parent_props_wrong_prop_kinds() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let parent_prop = create_prop_of_kind(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::String,
    )
    .await;
    let child_prop = create_prop_of_kind(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        PropKind::Object,
    )
    .await;

    let result = child_prop
        .set_parent_prop(&txn, &nats, &visibility, &history_actor, *parent_prop.id())
        .await;
    result.expect_err("should have errored, and it did not");
}
