use si_model::BillingAccount;
use si_model_test::{create_new_billing_account, one_time_setup, TestContext};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let ba = BillingAccount::new(&txn, &nats, "af", "adam and fletcher")
        .await
        .expect("cannot create billing account");
    assert_eq!(ba.name, "af");
    assert_eq!(ba.description, "adam and fletcher");
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let billing_account = create_new_billing_account(&txn, &nats).await;
    let ba = BillingAccount::get(&txn, &billing_account.id)
        .await
        .expect("cannot get billing account");
    assert_eq!(ba, billing_account);
}

#[tokio::test]
async fn get_by_name() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let billing_account = create_new_billing_account(&txn, &nats).await;
    let ba = BillingAccount::get_by_name(&txn, &billing_account.name)
        .await
        .expect("cannot get billing account by name");
    assert_eq!(ba, billing_account);
}

#[tokio::test]
async fn rotate_key_pair() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let billing_account = create_new_billing_account(&txn, &nats).await;
    BillingAccount::rotate_key_pair(&txn, &nats, billing_account.id)
        .await
        .expect("cannot rotate key pair");
}
