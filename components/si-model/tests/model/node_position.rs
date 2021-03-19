use si_model_test::{
    create_change_set, create_edit_session, create_entity_node, create_node_position,
    one_time_setup, signup_new_billing_account, TestContext,
};

use si_model::NodePosition;

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
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");

    let node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let context_id = "my-context";
    let x = "1.21";
    let y = "42001";

    let node_position =
        NodePosition::new(&txn, &nats, &node.id, context_id, x, y, &nba.workspace.id)
            .await
            .expect("cannot create new node position");

    assert!(node_position.id.starts_with("nodePosition:"));
    assert_eq!(node_position.node_id, node.id);
    assert_eq!(node_position.context_id, context_id);
    assert_eq!(node_position.x, x);
    assert_eq!(node_position.y, y);
}

#[tokio::test]
async fn create_or_update() {
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
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");

    let node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let existing_pos = create_node_position(&txn, &nats, &node.id, "context-1", &nba).await;

    let updated_pos = NodePosition::create_or_update(
        &txn,
        &nats,
        &existing_pos.node_id,
        &existing_pos.context_id,
        "4.5",
        "500",
        &nba.workspace.id,
    )
    .await
    .expect("cannot create or update node position");

    assert_eq!(existing_pos.id, updated_pos.id);
    assert_eq!(existing_pos.node_id, updated_pos.node_id);
    assert_eq!(existing_pos.context_id, updated_pos.context_id);
    assert_eq!(updated_pos.x, "4.5");
    assert_eq!(updated_pos.y, "500");

    let new_pos = NodePosition::create_or_update(
        &txn,
        &nats,
        &node.id,
        "newest-context",
        "7",
        "11",
        &nba.workspace.id,
    )
    .await
    .expect("cannot create or update node position");

    assert_ne!(existing_pos.id, new_pos.id);

    assert_eq!(new_pos.node_id, node.id);
    assert_eq!(new_pos.context_id, "newest-context");
    assert_eq!(new_pos.x, "7");
    assert_eq!(new_pos.y, "11");
}

#[tokio::test]
async fn by_node_id() {
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
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");

    let node_1 = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let node_2 = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let pos_1 = create_node_position(&txn, &nats, &node_1.id, "context-1", &nba).await;
    let pos_2 = create_node_position(&txn, &nats, &node_1.id, "context-2", &nba).await;
    let pos_3 = create_node_position(&txn, &nats, &node_1.id, "context-3", &nba).await;

    let pos_4 = create_node_position(&txn, &nats, &node_2.id, "context-4", &nba).await;

    let node_positions = NodePosition::get_by_node_id(&txn, &node_1.id)
        .await
        .expect("cannot get node positions");

    assert_eq!(node_positions.len(), 3);
    assert_eq!(true, node_positions.iter().any(|pos| pos == &pos_1));
    assert_eq!(true, node_positions.iter().any(|pos| pos == &pos_2));
    assert_eq!(true, node_positions.iter().any(|pos| pos == &pos_3));
    assert_eq!(false, node_positions.iter().any(|pos| pos == &pos_4));
}

#[tokio::test]
async fn save() {
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
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");

    let node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let mut pos = create_node_position(&txn, &nats, &node.id, "context-1", &nba).await;

    pos.x = "123".to_string();
    pos.y = "99.7".to_string();
    pos.save(&txn, &nats)
        .await
        .expect("cannot save node position");

    assert_eq!(pos.x, "123");
    assert_eq!(pos.y, "99.7");
}
