use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::{change_set, ListReply};

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::entity::create_entity;
use crate::models::system::create_system;
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
        .json(&change_set::CreateRequest {
            name: None,
            workspace_id: nba.workspace.id.clone(),
            organization_id: nba.organization.id.clone(),
        })
        .path("/changeSets")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "change set is created");
    let _change_set_reply: change_set::CreateReply =
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
        .method("PATCH")
        .header("authorization", &token)
        .json(&change_set::PatchRequest {
            op: change_set::PatchOps::Execute(change_set::ExecuteRequest { hypothetical: true }),
            workspace_id: nba.workspace.id.clone(),
            organization_id: nba.organization.id.clone(),
        })
        .path(&format!("/changeSets/{}", &change_set.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "change is executed");
    let _change_set_reply: change_set::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&change_set::PatchRequest {
            op: change_set::PatchOps::ExecuteWithAction(change_set::ExecuteWithActionRequest {
                node_id: entity.node_id.clone(),
                action: String::from("deploy"),
                system_id: system.id.clone(),
                edit_session_id: edit_session.id.clone(),
            }),
            workspace_id: nba.workspace.id.clone(),
            organization_id: nba.organization.id.clone(),
        })
        .path(&format!("/changeSets/{}", &change_set.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "change is executed");
    let _change_set_reply: change_set::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");
}

#[tokio::test]
async fn list_participants() {
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
    let _entity = create_entity(
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
        .path(&format!("/changeSetParticipants"))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.total_count, 3);
}
