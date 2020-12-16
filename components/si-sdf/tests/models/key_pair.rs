use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::{one_time_setup, TestContext};
use names::{Generator, Name};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{KeyPair, PublicKey};

pub async fn create_key_pair(txn: &PgTxn<'_>, nats: &NatsTxn, nba: &NewBillingAccount) -> KeyPair {
    let key_pair = KeyPair::new(
        txn,
        nats,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        nba.billing_account.id.clone(),
    )
    .await
    .expect("cannot create key pair");
    key_pair
}

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;

    let key_pair = KeyPair::new(&txn, &nats, "poop", &nba.billing_account.id)
        .await
        .expect("cannot create keypair");
    assert_eq!(key_pair.name, "poop");
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;

    let og_key_pair = KeyPair::new(&txn, &nats, "poop", &nba.billing_account.id)
        .await
        .expect("cannot create keypair");

    let key_pair = KeyPair::get(
        &txn,
        &og_key_pair.id,
        &og_key_pair.si_storable.billing_account_id,
    )
    .await
    .expect("cannot get keypair back");
    assert_eq!(key_pair, og_key_pair);
}

#[tokio::test]
async fn public_key_get_current() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;

    let _first_key_pair = KeyPair::new(&txn, &nats, "poop", &nba.billing_account.id)
        .await
        .expect("cannot create first keypair");
    let second_key_pair = KeyPair::new(&txn, &nats, "canoe", &nba.billing_account.id)
        .await
        .expect("cannot create second keypair");
    let pk = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");
    assert_eq!(pk, second_key_pair.into());
}
