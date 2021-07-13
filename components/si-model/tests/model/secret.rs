use si_model::{
    secret::EncryptedSecret, PublicKey, Secret, SecretAlgorithm, SecretKind, SecretObjectType,
    SecretVersion,
};
use si_model_test::{
    create_key_pair, create_secret, create_workspace, generate_fake_name, one_time_setup,
    signup_new_billing_account, TestContext,
};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let key_pair = create_key_pair(&txn, &nats, &nba).await;
    let name = generate_fake_name();

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
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_secret = create_secret(&txn, &nats, &nba.billing_account.id, &nba.workspace.id).await;

    let secret = Secret::get(&txn, &og_secret.id)
        .await
        .expect("cannot get secret back");
    assert_eq!(secret, og_secret);
}

#[tokio::test]
async fn secret_list_for_workspace() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let secret1 = create_secret(&txn, &nats, &nba.billing_account.id, &nba.workspace.id).await;
    let secret2 = create_secret(&txn, &nats, &nba.billing_account.id, &nba.workspace.id).await;

    let other_workspace = create_workspace(&txn, &nats, &nba).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let secret3 = create_secret(&txn, &nats, &nba.billing_account.id, &other_workspace.id).await;

    let reply = Secret::list_for_workspace(&txn, &nba.workspace.id)
        .await
        .expect("cannot list secrets");
    assert_eq!(reply.len(), 2);
    assert_eq!(true, reply.iter().any(|secret| secret == &secret1));
    assert_eq!(true, reply.iter().any(|secret| secret == &secret2));
    assert_eq!(false, reply.iter().any(|secret| secret == &secret3));
}

#[tokio::test]
async fn encrypted_secret_get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_secret = create_secret(&txn, &nats, &nba.billing_account.id, &nba.workspace.id).await;

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
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");
    let name = generate_fake_name();

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
