use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::{EventLog, GetReply, ListReply};

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::models::event_log::create_event_log;
use crate::one_time_setup;
use crate::TestContext;

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let event_log = create_event_log(&pg, &nats_conn, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/eventLogs/{}", &event_log.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let _item: EventLog =
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
    let nba = signup_new_billing_account(&txn, &nats).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let _event_log = create_event_log(&pg, &nats_conn, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/eventLogs")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.total_count, 1);
}
