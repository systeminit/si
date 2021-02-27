use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};

use si_model::{EditSession, Entity};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = create_change_set(&txn, &nats, &nba).await;

    let edit_session = EditSession::new(
        &txn,
        &nats,
        Some("floopy boodles".to_string()),
        change_set.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create edit session");

    assert_eq!(&edit_session.name, "floopy boodles");
}

#[tokio::test]
async fn save_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
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
    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");
    let edit_session_entity =
        Entity::for_edit_session(&txn, &node.object_id, &change_set.id, &edit_session.id)
            .await
            .expect("cannot get entity for edit session");
    let change_set_entity = Entity::for_change_set(&txn, &node.object_id, &change_set.id)
        .await
        .expect("cannot get entity for change set");
    assert_eq!(edit_session_entity, change_set_entity);

    // We test it twice, because we need to make sure the upsert works!
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    edit_session_entity
        .save_for_edit_session(&txn, &change_set.id, &edit_session.id)
        .await
        .expect("cannot save entity for edit session again");
    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session again");
    let change_set_entity = Entity::for_change_set(&txn, &node.object_id, &change_set.id)
        .await
        .expect("cannot get entity for change set");
    assert_eq!(edit_session_entity, change_set_entity);
}
