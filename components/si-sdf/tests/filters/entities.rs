use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::{entity::GetReply, ListReply};

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::system::create_system;
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let system = create_system(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let entity = create_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/entities/{}", &entity.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be found");
    let _reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let system = create_system(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let mut entity = create_entity(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &system,
    )
    .await;
    entity
        .save_head(&txn, &nats)
        .await
        .expect("cannot save head entity");
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/entities")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.total_count, 1);
}
