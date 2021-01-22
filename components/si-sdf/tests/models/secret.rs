use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::models::key_pair::create_key_pair;
use crate::{one_time_setup, TestContext};
use names::{Generator, Name};
use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{
    secret::EncryptedSecret, PublicKey, Secret, SecretAlgorithm, SecretKind, SecretObjectType,
    SecretVersion,
};

pub async fn create_secret(txn: &PgTxn<'_>, nats: &NatsTxn, nba: &NewBillingAccount) -> Secret {
    let key_pair = create_key_pair(txn, nats, nba).await;

    let secret = Secret::new(
        txn,
        nats,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        key_pair.id,
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create secret");
    secret
}

pub async fn encrypt_message(
    txn: &PgTxn<'_>,
    nba: &NewBillingAccount,
    message: &serde_json::Value,
) -> Vec<u8> {
    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize"),
        &public_key.public_key,
    );
    crypted
}

pub async fn create_secret_with_message(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    message: serde_json::Value,
) -> Secret {
    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize"),
        &public_key.public_key,
    );

    let secret = Secret::new(
        txn,
        nats,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        crypted,
        public_key.id,
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create secret");
    secret
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
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let key_pair = create_key_pair(&txn, &nats, &nba).await;
    let name = Generator::with_naming(Name::Numbered).next().unwrap();

    let secret = Secret::new(
        &txn,
        &nats,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        "im-a-secret",
        key_pair.id,
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create secret");

    assert_eq!(&secret.name, &name);
    assert_eq!(&secret.object_type, &SecretObjectType::Credential);
    assert_eq!(&secret.kind, &SecretKind::DockerHub);
    assert_eq!(&secret.si_storable.workspace_id, &nba.workspace.id);
}

#[tokio::test]
async fn secret_get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_secret = create_secret(&txn, &nats, &nba).await;

    let secret = Secret::get(&txn, &og_secret.id)
        .await
        .expect("cannot get secret back");
    assert_eq!(secret, og_secret);
}

#[tokio::test]
async fn secret_list() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let _secret1 = create_secret(&txn, &nats, &nba).await;
    let _secret2 = create_secret(&txn, &nats, &nba).await;

    let reply = Secret::list(&txn, &nba.billing_account.id, None, None, None, None, None)
        .await
        .expect("cannot list secrets");
    assert_eq!(reply.items.len(), 2);
}

#[tokio::test]
async fn encrypted_secret_get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_secret = create_secret(&txn, &nats, &nba).await;

    let encrypted_secret = EncryptedSecret::get(&txn, &og_secret.id)
        .await
        .expect("cannot get secret back");

    assert_eq!(&encrypted_secret.id, &og_secret.id);
    assert_eq!(&encrypted_secret.name, &og_secret.name);
    assert_eq!(&encrypted_secret.object_type, &og_secret.object_type);
    assert_eq!(&encrypted_secret.kind, &og_secret.kind);
    assert_eq!(&encrypted_secret.si_storable, &og_secret.si_storable);
}

#[tokio::test]
async fn encrypt_decrypt_round_trip() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");
    let name = Generator::with_naming(Name::Numbered).next().unwrap();

    let message = serde_json::json!({"song": "I'm a little teapot"});
    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize"),
        &public_key.public_key,
    );

    let secret = Secret::new(
        &txn,
        &nats,
        &name,
        SecretObjectType::Credential,
        SecretKind::AwsAccessKey,
        crypted,
        public_key.id,
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create secret");

    let decrypted = EncryptedSecret::get(&txn, &secret.id)
        .await
        .expect("cannot get secret")
        .decrypt(&txn)
        .await
        .expect("cannot decrypt secret");

    assert_eq!(&decrypted.message, &message);
}
