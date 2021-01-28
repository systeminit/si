use warp::http::StatusCode;

use si_sdf::filters::api;
use si_sdf::models::GetReply;

use crate::filters::users::login_user;
use crate::models::billing_account::signup_new_billing_account;
use crate::models::change_set::create_change_set;
use crate::models::edit_session::create_edit_session;
use crate::models::node::create_entity_node;
use crate::models::system::create_system;
use crate::one_time_setup;
use crate::TestContext;

use si_sdf::models::{edge, Edge, ListReply};

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
    let entity_node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    let entity_node_second = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    entity_node
        .configured_by(&txn, &nats, &entity_node_second.id)
        .await
        .expect("cannot configure node");
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let edge = Edge::by_kind_and_tail_node_id(
        &txn,
        &si_sdf::models::EdgeKind::Configures,
        &entity_node_second.id,
    )
    .await
    .expect("cannot get edge")
    .pop()
    .expect("no edge returned");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/edges/{}", &edge.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let _item: Edge =
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
    let entity_node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    let entity_node_second = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    entity_node
        .configured_by(&txn, &nats, &entity_node_second.id)
        .await
        .expect("cannot configure node");
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path("/edges")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: ListReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.total_count, 3);
}

#[tokio::test]
async fn delete() {
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
    let entity_node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    let entity_node_second = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    entity_node
        .configured_by(&txn, &nats, &entity_node_second.id)
        .await
        .expect("cannot configure node");
    txn.commit().await.expect("cannot commit txn");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let edge = Edge::by_kind_and_tail_node_id(
        &txn,
        &si_sdf::models::EdgeKind::Configures,
        &entity_node_second.id,
    )
    .await
    .expect("cannot get edge")
    .pop()
    .expect("no edge returned");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("DELETE")
        .header("authorization", &token)
        .path(format!("/edges/{}", &edge.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be deleted");
    let _reply: edge::DeleteReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
}

#[tokio::test]
async fn all_predecessors() {
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
    let entity_node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    let entity_node_second = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    entity_node
        .configured_by(&txn, &nats, &entity_node_second.id)
        .await
        .expect("cannot configure node");
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allPredecessors?nodeId={}&edgeKind=configures",
            &entity_node.id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllPredecessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 1);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allPredecessors?nodeId={}&edgeKind=includes",
            &entity_node.id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllPredecessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 1);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allPredecessors?objectId={}&edgeKind=includes",
            &entity_node.object_id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllPredecessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 1);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allPredecessors?objectId={}&edgeKind=configures",
            &entity_node.object_id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllPredecessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 1);
}

#[tokio::test]
async fn all_successors() {
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
    let entity_node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    let entity_node_second = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &system,
        &change_set,
        &edit_session,
    )
    .await;
    entity_node
        .configured_by(&txn, &nats, &entity_node_second.id)
        .await
        .expect("cannot configure node");
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allSuccessors?nodeId={}&edgeKind=configures",
            &entity_node_second.id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllSuccessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 1);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allSuccessors?nodeId={}&edgeKind=includes",
            &system.node_id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllSuccessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 2);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allSuccessors?objectId={}&edgeKind=configures",
            &entity_node_second.object_id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllSuccessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 1);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/edges/allSuccessors?objectId={}&edgeKind=includes",
            &system.id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "list model should succeed");
    let reply: edge::AllSuccessorsReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    assert_eq!(reply.edges.len(), 2);
}
