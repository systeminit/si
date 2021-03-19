use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};

use si_model::Entity;

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;

    assert_eq!(&node.object_id.starts_with("entity:"), &true);
    assert_eq!(&node.object_type, "service");
}

#[tokio::test]
async fn for_edit_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "leftHandPath",
    )
    .await;

    let entity = Entity::for_edit_session(&txn, &node.object_id, &change_set.id, &edit_session.id)
        .await
        .expect("cannot get entity for edit session");
    dbg!(&entity);
    assert_eq!(entity.entity_type, "leftHandPath");
    assert_eq!(entity.name, entity.description);
    assert_eq!(
        &entity.properties,
        &serde_json::json!({"baseline": { "simpleString": "chunky" } })
    );
}
