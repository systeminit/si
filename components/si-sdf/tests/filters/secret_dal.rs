use crate::{
    filters::session_dal::login_user, models::billing_account::signup_new_billing_account,
    models::secret::encrypt_message, one_time_setup, TestContext,
};
use si_sdf::{
    filters::api,
    handlers::secret_dal::{CreateSecretReply, CreateSecretRequest, GetPublicKeyReply},
    models::{PublicKey, SecretAlgorithm, SecretKind, SecretObjectType, SecretVersion},
};
use warp::http::StatusCode;

#[tokio::test]
async fn get_public_key() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/secretDal/getPublicKey")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be created");
    let reply: GetPublicKeyReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    let expected_public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get public key");

    assert_eq!(expected_public_key, reply.public_key);
}

#[tokio::test]
async fn create_secret() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get public key");

    let message = serde_json::json![{"rise":"again we will rise"}];
    let encrypted_message = encrypt_message(&txn, &nba, &message).await;

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .json(&CreateSecretRequest {
            name: "my-fear".to_string(),
            object_type: SecretObjectType::Credential,
            kind: SecretKind::DockerHub,
            crypted: encrypted_message,
            key_pair_id: public_key.id.clone(),
            version: SecretVersion::V1,
            algorithm: SecretAlgorithm::Sealedbox,
            workspace_id: nba.workspace.id.clone(),
        })
        .path("/secretDal/createSecret")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be created");
    let reply: CreateSecretReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    assert_eq!("my-fear", reply.secret.name);
    assert_eq!(SecretObjectType::Credential, reply.secret.object_type);
    assert_eq!(SecretKind::DockerHub, reply.secret.kind);
}
