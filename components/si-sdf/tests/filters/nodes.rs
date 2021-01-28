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

use si_sdf::models::{node, ops, Entity, Node};

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
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .json(&node::CreateRequest {
            name: None,
            kind: node::NodeKind::Entity,
            object_type: "service".into(),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            system_ids: Some(vec![system.id.clone()]),
        })
        .path("/nodes")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let _node_reply: node::CreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
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
    let second_system = create_system(
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
    let second_entity_node = create_entity_node(
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
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    // Include System
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::PatchRequest {
            op: node::PatchOp::IncludeSystem(node::PatchIncludeSystemRequest {
                system_id: second_system.id.clone(),
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .path(&format!("/nodes/{}", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let patch_reply: node::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    match patch_reply {
        node::PatchReply::IncludeSystem(_) => {}
        r => {
            panic!("wrong reply for node patch request; r={:?}", r);
        }
    }

    // Configured By
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::PatchRequest {
            op: node::PatchOp::ConfiguredBy(node::PatchConfiguredByRequest {
                node_id: second_entity_node.id.clone(),
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .path(&format!("/nodes/{}", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let patch_reply: node::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    match patch_reply {
        node::PatchReply::ConfiguredBy(_) => {}
        r => {
            panic!("wrong reply for node patch request; r={:?}", r);
        }
    }

    // Set Position
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::PatchRequest {
            op: node::PatchOp::SetPosition(node::PatchSetPositionRequest {
                context: String::from("darkness"),
                position: node::Position::new(42, 42),
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .path(&format!("/nodes/{}", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let patch_reply: node::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    match patch_reply {
        node::PatchReply::SetPosition(_) => {}
        r => {
            panic!("wrong reply for node patch request; r={:?}", r);
        }
    }

    // Sync Resource
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::PatchRequest {
            op: node::PatchOp::SyncResource(node::SyncResourceRequest {
                system_id: system.id.clone(),
                change_set_id: Some(change_set.id.clone()),
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
        })
        .path(&format!("/nodes/{}", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let patch_reply: node::PatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    match patch_reply {
        node::PatchReply::SyncResource(_) => {}
        r => {
            panic!("wrong reply for node patch request; r={:?}", r);
        }
    }
}

#[tokio::test]
async fn object_patch() {
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
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    // Entity Set
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::ObjectPatchRequest {
            op: ops::OpRequest::EntitySet(ops::OpEntitySetRequest {
                path: vec![String::from("laid to waste")],
                value: serde_json::json!["lamb of god!"],
                override_system: None,
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
        })
        .path(&format!("/nodes/{}/object", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let _patch_reply: node::ObjectPatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    // Set name
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::ObjectPatchRequest {
            op: ops::OpRequest::NameSet(ops::OpSetNameRequest {
                value: String::from("poop canoe"),
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
        })
        .path(&format!("/nodes/{}/object", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let _patch_reply: node::ObjectPatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    // Entity Action
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::ObjectPatchRequest {
            op: ops::OpRequest::EntityAction(ops::OpEntityActionRequest {
                action: String::from("deploy"),
                system_id: system.id.clone(),
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
        })
        .path(&format!("/nodes/{}/object", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let _patch_reply: node::ObjectPatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    // Entity Delete
    let res = warp::test::request()
        .method("PATCH")
        .header("authorization", &token)
        .json(&node::ObjectPatchRequest {
            op: ops::OpRequest::EntityDelete(ops::OpEntityDeleteRequest {
                // This is total, absolute horseshit.
                cascade: false,
            }),
            organization_id: nba.organization.id.clone(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
        })
        .path(&format!("/nodes/{}/object", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let _patch_reply: node::ObjectPatchReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
}

#[tokio::test]
async fn get_object() {
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
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    // Get Projection
    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!(
            "/nodes/{}/object?changeSetId={}",
            &entity_node.id, &change_set.id
        ))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should succeed");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    let entity: Entity = serde_json::from_value(reply.item).expect("cannot deserialize object");
    assert_eq!(&entity.base, &false);
    assert_eq!(&entity.head, &false);

    // Get Head when not saved
    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!("/nodes/{}/object", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND, "patch should fail");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let mut pe = entity_node
        .get_projection_object_entity(&txn, &change_set.id)
        .await
        .expect("cannot get projection");
    pe.save_head(&txn, &nats)
        .await
        .expect("cannot save head object");
    txn.commit().await.expect("cannot commit txn");
    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(&format!("/nodes/{}/object", &entity_node.id))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "patch should fail");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");
    let entity: Entity = serde_json::from_value(reply.item).expect("cannot deserialize object");
    assert_eq!(&entity.base, &false);
    assert_eq!(&entity.head, &true);
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
    txn.commit().await.expect("cannot commit txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &token)
        .path(format!("/nodes/{}", &entity_node.id).as_ref())
        .reply(&filter)
        .await;
    assert_eq!(res.status(), StatusCode::OK, "model should be found");
    let reply: GetReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize get model reply");
    let _item: Node =
        serde_json::from_value(reply.item).expect("cannot deserialize model from get model reply");
}
