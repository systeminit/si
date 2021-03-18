use si_model_test::{
    create_new_billing_account, create_test_organization, one_time_setup, TestContext,
};

use si_model::Workspace;

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = create_new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspace = Workspace::new(&txn, &nats, "jesse leach", &ba.id, &org.id)
        .await
        .expect("cannot create workspace");
    assert_eq!(workspace.name, "jesse leach");
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = create_new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspace = Workspace::new(&txn, &nats, "adam d", &ba.id, &org.id)
        .await
        .expect("cannot create workspace");
    assert_eq!(workspace.name, "adam d");
    let wg = Workspace::get(&txn, &workspace.id)
        .await
        .expect("cannot get workspace");
    assert_eq!(wg.name, workspace.name);
    assert_eq!(wg.id, workspace.id);
}

#[tokio::test]
async fn save() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = create_new_billing_account(&txn, &nats).await;
    let org = create_test_organization(&txn, &nats, "dark tranquility", &ba.id).await;

    let workspace = Workspace::new(&txn, &nats, "adam d", &ba.id, &org.id)
        .await
        .expect("cannot create workspace");
    assert_eq!(workspace.name, "adam d");
    let mut wg = Workspace::get(&txn, &workspace.id)
        .await
        .expect("cannot get workspace");
    assert_eq!(wg.name, workspace.name);
    assert_eq!(wg.id, workspace.id);
    wg.name = String::from("poopy pants");
    let updated_wg = wg.save(&txn, &nats).await.expect("cannot save workspace");
    assert_eq!(&updated_wg.name, "poopy pants");
    let wg_updated = Workspace::get(&txn, &workspace.id)
        .await
        .expect("cannot get workspace");
    assert_eq!(&wg_updated.name, "poopy pants");
}
