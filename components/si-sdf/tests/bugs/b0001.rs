use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::ops::{
    create_op_entity_action, create_op_entity_delete, create_op_entity_set, create_op_set_name,
};
use crate::models::system::create_system;
use crate::{one_time_setup, TestContext};

#[tokio::test]
async fn node_editing_fails_after_change_set_execution() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit().await.expect("failed to commit txn");

    let txn = conn.transaction().await.expect("cannot create txn");
    let system = create_system(
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

    let entity = create_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;
    txn.commit().await.expect("failed to commit txn");

    let txn = conn.transaction().await.expect("cannot create txn");
    change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, false, None)
        .await
        .expect("failed to execute change set");
    txn.commit().await.expect("failed to commit txn");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut second_change_set = create_change_set(&txn, &nats, &nba).await;
    let second_edit_session = create_edit_session(&txn, &nats, &nba, &second_change_set).await;
    txn.commit().await.expect("failed to commit txn");

    let txn = conn.transaction().await.expect("cannot create txn");
    create_op_set_name(
        &txn,
        &nats,
        &nba,
        &second_change_set,
        &second_edit_session,
        &entity,
        "elton john",
    )
    .await;
    second_change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .expect("failed to execute second change set - this is the bug!");
    txn.commit().await.expect("failed to commit txn");
}
