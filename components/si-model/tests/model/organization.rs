use si_model::Organization;
use si_model::test::{create_new_billing_account, one_time_setup, TestContext};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let ba = create_new_billing_account(&txn, &nats).await;

    let organization = Organization::new(&txn, &nats, "jesse leach", &ba.id)
        .await
        .expect("cannot create organization");
    assert_eq!(organization.name, "jesse leach");
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = create_new_billing_account(&txn, &nats).await;

    let organization = Organization::new(&txn, &nats, "adam d", &ba.id)
        .await
        .expect("cannot create organization");
    assert_eq!(organization.name, "adam d");
    let wg = Organization::get(&txn, &organization.id)
        .await
        .expect("cannot get organization");
    assert_eq!(wg.name, organization.name);
    assert_eq!(wg.id, organization.id);
}

#[tokio::test]
async fn save() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let ba = create_new_billing_account(&txn, &nats).await;
    let organization = Organization::new(&txn, &nats, "adam d", &ba.id)
        .await
        .expect("cannot create organization");
    assert_eq!(organization.name, "adam d");
    let mut wg = Organization::get(&txn, &organization.id)
        .await
        .expect("cannot get organization");
    assert_eq!(wg.name, organization.name);
    assert_eq!(wg.id, organization.id);
    wg.name = String::from("poopy pants");
    let updated_wg = wg
        .save(&txn, &nats)
        .await
        .expect("cannot save organization");
    assert_eq!(&updated_wg.name, "poopy pants");
    let wg_updated = Organization::get(&txn, &organization.id)
        .await
        .expect("cannot get organization");
    assert_eq!(&wg_updated.name, "poopy pants");
}
