use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::edit_session;

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
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
    let change_set = create_change_set(&txn, &nats, &nba).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .json(&edit_session::CreateRequest {
            name: None,
            workspace_id: nba.workspace.id.clone(),
            organization_id: nba.organization.id.clone(),
        })
        .path(&format!("/changeSets/{}/editSessions", &change_set.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "edit session is created");
    let _reply: edit_session::CreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");
}

#[tokio::test]
async fn patch() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&edit_session::PatchRequest::Cancel(true))
        .path(&format!(
            "/changeSets/{}/editSessions/{}",
            &change_set.id, &edit_session.id
        ))
        .reply(&filter)
        .await;
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "edit session patch is executed"
    );
    let _reply: edit_session::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");
}
