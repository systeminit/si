use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::api_client::{ApiClient, ApiClientKind, CreateReply, CreateRequest};
use si_sdf::models::{GetReply, ListReply};

use crate::filters::users::login_user;
use crate::models::api_client::create_api_client;
use crate::models::billing_account::signup_new_billing_account;
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

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .json(&CreateRequest {
            name: String::from("killer be killed"),
            kind: ApiClientKind::Cli,
        })
        .path(format!("/apiClients").as_ref())
        .reply(&filter)
        .await;

    let _result: CreateReply = serde_json::from_str(String::from_utf8_lossy(res.body()).as_ref())
        .expect("cannot create an api client, results do not deserialize");
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
    let (api_client, _token) = create_api_client(&txn, &nats, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/apiClients/{}", &api_client.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let _item: ApiClient =
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
    let (_api_client, _token) = create_api_client(&txn, &nats, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/apiClients")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.total_count, 1);
}
