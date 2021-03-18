use crate::{
    filters::{application_context_dal::create_application, session_dal::login_user},
    generate_fake_name,
    models::{
        billing_account::signup_new_billing_account, change_set::create_change_set,
        edit_session::create_edit_session, node::create_entity_node,
        node_position::create_node_position,
    },
    one_time_setup, TestContext,
};
use si_sdf::{
    filters::api,
    handlers::editor_dal::{
        NodeCreateForApplicationRequest, NodeCreateReply, UpdateNodePositionReply,
        UpdateNodePositionRequest,
    },
    models::{Entity, Node, NodeKind, NodePosition},
};
use warp::http::StatusCode;

#[tokio::test]
async fn node_create_for_application() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let application = create_application(&ctx, &nba).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let name = generate_fake_name();

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/editorDal/nodeCreateForApplication")
        .json(&NodeCreateForApplicationRequest {
            name: Some(name.clone()),
            kind: NodeKind::Entity,
            object_type: "Service".to_string(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            system_id: nba.system.id.clone(),
            application_id: application.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: NodeCreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let reply_entity = reply.object.entity.expect("node object was not an entity");

    let expected_node = Node::get(&txn, &reply.node.id)
        .await
        .expect("cannot get node");
    let expected_entity = Entity::get_projection(&txn, &reply_entity.id, &change_set.id)
        .await
        .expect("cannot get entity");

    assert_eq!(expected_node, reply.node);
    assert_eq!(expected_entity, reply_entity);
}

#[tokio::test]
async fn update_node_position() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, secret_key) = ctx.entries();
    let mut conn = pg.pool.get().await.expect("cannot get connection");
    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &nba.system,
        &change_set,
        &edit_session,
    )
    .await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/editorDal/updateNodePosition")
        .json(&UpdateNodePositionRequest {
            node_id: node.id.clone(),
            context_id: "context-1".to_string(),
            x: "12".to_string(),
            y: "-17.3369".to_string(),
            workspace_id: nba.workspace.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "update should succeed");
    let reply: UpdateNodePositionReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let expected_node_position = NodePosition::get_by_node_id(&txn, &node.id)
        .await
        .expect("could not get node positions")
        .pop()
        .expect("could not get last and only node position");

    assert_eq!(expected_node_position, reply.node_position);

    txn.commit().await.expect("cannot commit txn");

    // Now try a node with 2 existing contexts with an update to 1

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let node = create_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &nba.system,
        &change_set,
        &edit_session,
    )
    .await;
    let context_id = "context-cool".to_string();
    create_node_position(&txn, &nats, &node.id, &context_id, &nba).await;
    let other_pos = create_node_position(&txn, &nats, &node.id, "dont-touch-me", &nba).await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/editorDal/updateNodePosition")
        .json(&UpdateNodePositionRequest {
            node_id: node.id.clone(),
            context_id: context_id.clone(),
            x: "23.2".to_string(),
            y: "0".to_string(),
            workspace_id: nba.workspace.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "update should succeed");
    let reply: UpdateNodePositionReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let positions = NodePosition::get_by_node_id(&txn, &node.id)
        .await
        .expect("could not get node positions");

    assert_eq!(2, positions.len());
    assert!(positions
        .iter()
        .any(|expected| expected == &reply.node_position));
    assert!(positions.iter().any(|expected| expected == &other_pos));
}
