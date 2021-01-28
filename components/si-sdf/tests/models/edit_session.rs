use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::change_set::create_change_set;
use crate::models::entity::create_entity;
use crate::models::ops::create_op_set_name;
use crate::models::system::create_system;
use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{ops::OpSetName, ChangeSet, EditSession};

pub async fn create_edit_session(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
) -> EditSession {
    EditSession::new(
        &txn,
        &nats,
        None,
        change_set.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("Cannot create new edit session")
}

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
    assert_eq!(&edit_session.reverted, &false);
}

#[tokio::test]
async fn cancel() {
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
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");
    edit_session
        .cancel(&pg, &txn, &nats_conn, &nats, &veritech, None)
        .await
        .expect("cannot cancel empty edit session");

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
    let op_set_name = create_op_set_name(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &entity,
        "eric clapton",
    )
    .await;

    edit_session
        .cancel(&pg, &txn, &nats_conn, &nats, &veritech, None)
        .await
        .expect("cannot cancle edit session");

    let row = txn
        .query_one(
            "SELECT obj AS object FROM ops WHERE si_id = $1",
            &[&op_set_name.id],
        )
        .await
        .expect("cannot get op");
    let op_set_name_skipped_json: serde_json::Value =
        row.try_get("object").expect("cannot get object for row");
    let op_set_name_skipped: OpSetName =
        serde_json::from_value(op_set_name_skipped_json).expect("cannot make object from json");
    assert_eq!(op_set_name_skipped.si_op.skip, true);
}
