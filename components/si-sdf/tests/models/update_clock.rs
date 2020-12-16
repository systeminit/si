use crate::models::billing_account::signup_new_billing_account;
use crate::{one_time_setup, TestContext};

#[tokio::test]
async fn updates_a_clock() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit().await.expect("cannot commit transaction");

    let update_clock = si_sdf::models::update_clock::next_update_clock(&nba.workspace.id)
        .await
        .expect("canot get update clock");
    assert_eq!(update_clock.epoch, 0);
    assert_eq!(update_clock.update_count, 1);
}
