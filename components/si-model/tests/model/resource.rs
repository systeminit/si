use si_model::Resource;
use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
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

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let resource = Resource::new(
        &pg,
        &nats_conn,
        serde_json::json!({ "foo": "bar" }),
        &service_node.object_id,
        &system_node.object_id,
        &nba.workspace.id,
    )
    .await
    .expect("resource is created");

    assert_eq!(&resource.system_id, &system_node.object_id);
    assert_eq!(&resource.entity_id, &service_node.object_id);
}

#[tokio::test]
async fn get_by_entity_and_system() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
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

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let created = Resource::new(
        &pg,
        &nats_conn,
        serde_json::json!({ "foo": "bar" }),
        &service_node.object_id,
        &system_node.object_id,
        &nba.workspace.id,
    )
    .await
    .expect("resource is created");

    let txn = conn.transaction().await.expect("cannot create txn");

    let resource =
        Resource::get_by_entity_and_system(&txn, &service_node.object_id, &system_node.object_id)
            .await
            .expect("failed to query for the resource for system")
            .expect("could not find resource for system");

    assert_eq!(created, resource);
}

#[tokio::test]
async fn get_by_entity_and_system_nonexistant() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
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

    let resource =
        Resource::get_by_entity_and_system(&txn, &service_node.object_id, &system_node.object_id)
            .await
            .expect("failed to query for the resource for system");

    assert_eq!(None, resource);
}

#[tokio::test]
async fn await_sync() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

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

    edit_session
        .save_session(&txn)
        .await
        .expect("failed to save edit session");
    change_set
        .apply(&txn)
        .await
        .expect("failed to apply change set");

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let mut resource = Resource::new(
        &pg,
        &nats_conn,
        serde_json::json!({ "foo": "bar" }),
        &service_node.object_id,
        &system_node.object_id,
        &nba.workspace.id,
    )
    .await
    .expect("resource is created");

    let updated_at_created = resource.timestamp.clone();

    resource
        .await_sync(pg.clone(), nats_conn.clone(), veritech.clone())
        .await
        .expect("failed to sync resource");

    assert_ne!(updated_at_created, resource.timestamp);
}
