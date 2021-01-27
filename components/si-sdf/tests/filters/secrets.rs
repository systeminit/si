use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::{
    secret, GetReply, ListReply, PublicKey, Secret, SecretAlgorithm, SecretKind, SecretObjectType,
    SecretVersion,
};

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::models::secret::{create_secret, encrypt_message};
use crate::one_time_setup;
use crate::TestContext;

#[tokio::test]
async fn create() {
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
        .json(&secret::CreateRequest {
            name: "my-fear".to_string(),
            object_type: SecretObjectType::Credential,
            kind: SecretKind::DockerHub,
            crypted: encrypted_message,
            key_pair_id: public_key.id.clone(),
            version: SecretVersion::V1,
            algorithm: SecretAlgorithm::Sealedbox,
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .path("/secrets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "model should be created");
    let _reply: secret::CreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize create secret reply");
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let secret = create_secret(&txn, &nats, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/secrets/{}", &secret.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let _item: Secret =
        serde_json::from_value(reply.item).expect("cannot deserialize model from get model reply");
}

#[tokio::test]
async fn list() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let _secret = create_secret(&txn, &nats, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/secrets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.total_count, 1);
}
