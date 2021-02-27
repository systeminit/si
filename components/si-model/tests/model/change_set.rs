use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, one_time_setup,
    signup_new_billing_account, TestContext,
};

use si_model::{ChangeSet, ChangeSetStatus, Entity};

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

    let change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy mcpants".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    assert_eq!(&change_set.name, "poopy mcpants");
    assert_eq!(&change_set.status, &ChangeSetStatus::Open);
}

#[tokio::test]
async fn get() {
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

    let og_change_set = ChangeSet::new(
        &txn,
        &nats,
        Some("poopy mcpants".to_string()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create change_set");

    let change_set = ChangeSet::get(&txn, &og_change_set.id)
        .await
        .expect("cannot get change set");
    assert_eq!(&og_change_set, &change_set);
}

#[tokio::test]
async fn apply() {
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

    let mut change_set = create_change_set(&txn, &nats, &nba).await;
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
    change_set
        .apply(&txn)
        .await
        .expect("cannot save change set");
    let change_set_entity = Entity::for_change_set(&txn, &node.object_id, &change_set.id)
        .await
        .expect("cannot get entity for change set");
    let head_entity = Entity::for_head(&txn, &node.object_id)
        .await
        .expect("cannot get entity for head");

    assert_eq!(change_set_entity, head_entity);

    // We test it twice, because we need to make sure the upsert works!
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let edit_session_entity =
        Entity::for_edit_session(&txn, &node.object_id, &change_set.id, &edit_session.id)
            .await
            .expect("cannot get entity for edit session");
    edit_session_entity
        .save_for_edit_session(&txn, &change_set.id, &edit_session.id)
        .await
        .expect("cannot save entity for edit session again");
    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session again");
    change_set
        .apply(&txn)
        .await
        .expect("cannot save change set");
    let change_set_entity = Entity::for_change_set(&txn, &node.object_id, &change_set.id)
        .await
        .expect("cannot get entity for change set");
    let head_entity = Entity::for_head(&txn, &node.object_id)
        .await
        .expect("cannot get entity for head");
    assert_eq!(change_set_entity, head_entity);
}
