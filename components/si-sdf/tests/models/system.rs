use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::node::create_system_node;
use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsTxn, PgPool, PgTxn};
use si_sdf::models::{ChangeSet, ChangeSetParticipant, EditSession, System};
use si_sdf::veritech::Veritech;

pub async fn create_system(
    pool: &PgPool,
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> System {
    let node = create_system_node(
        &pool,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let system = node
        .get_projection_object_system(&txn, &change_set.id)
        .await
        .expect("cannot get system projection");
    system
}

#[tokio::test]
async fn new() {
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    assert_eq!(&system.base, &false);
    assert_eq!(&system.head, &false);
    assert_eq!(&system.node_id.starts_with("node:"), &true);

    let csp_exists = ChangeSetParticipant::exists(&txn, &change_set.id, &system.id)
        .await
        .expect("cannot check if system is in change set participants");
    assert_eq!(&csp_exists, &true);
}

#[tokio::test]
async fn save_head() {
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    system
        .save_head(&txn, &nats)
        .await
        .expect("failed to save system as head");
    assert_eq!(&system.head, &true);
    assert_eq!(&system.base, &false);
    assert_eq!(&system.si_change_set.is_some(), &false);
}

#[tokio::test]
async fn save_projection() {
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    system
        .save_projection(&txn, &nats)
        .await
        .expect("failed to save system as head");
    assert_eq!(&system.head, &false);
    assert_eq!(&system.base, &false);
    assert_eq!(&system.si_change_set.is_some(), &true);
}

#[tokio::test]
async fn get_any() {
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut og_system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let system = System::get_any(&txn, &og_system.id)
        .await
        .expect("cannot get any system");
    assert_eq!(&og_system.id, &system.id);
    assert_eq!(&system.base, &true);

    og_system
        .save_head(&txn, &nats)
        .await
        .expect("cannot save system as head");
    let system = System::get_any(&txn, &og_system.id)
        .await
        .expect("cannot get any system");
    assert_eq!(&og_system.id, &system.id);
    assert_eq!(&system.head, &true);
    assert_eq!(&system.base, &false);
}

#[tokio::test]
async fn get_head() {
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let mut og_system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let system = System::get_head(&txn, &og_system.id).await;
    assert_eq!(&system.is_err(), &true);

    og_system
        .save_head(&txn, &nats)
        .await
        .expect("cannot save system as head");
    let system = System::get_head(&txn, &og_system.id)
        .await
        .expect("cannot get any system");
    assert_eq!(&og_system.id, &system.id);
    assert_eq!(&system.head, &true);
    assert_eq!(&system.base, &false);
}

#[tokio::test]
async fn get_projection() {
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit()
        .await
        .expect("failed to commit the new change set");

    let txn = conn.transaction().await.expect("cannot create txn");
    let og_system = create_system(
        &pg,
        &txn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let system = System::get_projection(&txn, &og_system.id, &change_set.id)
        .await
        .expect("cannot get system projection");
    assert_eq!(&og_system.id, &system.id);
    assert_eq!(&system.head, &false);
    assert_eq!(&system.base, &false);
    assert_eq!(&system.si_change_set.is_some(), &true);
}
