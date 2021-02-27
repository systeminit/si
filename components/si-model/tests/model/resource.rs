use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};

use si_model::Resource;

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

    let system_node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "system",
    )
    .await;
    let service_node = create_custom_node(
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

    let resource = Resource::new(
        &txn,
        &nats,
        serde_json::json!({ "foo": "bar" }),
        &system_node.object_id,
        &service_node.id,
        &service_node.object_id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("resource is created");

    assert_eq!(&resource.node_id, &service_node.id);
    assert_eq!(&resource.entity_id, &service_node.object_id);
}

#[tokio::test]
async fn for_edit_session_by_entity_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let system_node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "system",
    )
    .await;
    let service_node = create_custom_node(
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

    let resource = Resource::new(
        &txn,
        &nats,
        serde_json::json!({ "foo": "bar" }),
        &system_node.object_id,
        &service_node.id,
        &service_node.object_id,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("resource is created");

    let resources = Resource::for_edit_session_by_entity_id(
        &txn,
        &service_node.object_id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot get list of resources for edit session");

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0], resource);
}
