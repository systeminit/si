use si_model::{Edge, EdgeKind, Node, NodePosition, SchematicKind, Vertex};
use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, create_entity_node,
    create_node_position, generate_fake_name, one_time_setup, signup_new_billing_account,
    TestContext,
};

use crate::filters::{application_context_dal::create_application, session_dal::login_user};
use si_sdf::{
    filters::api,
    handlers::schematic_dal::{
        Connection, ConnectionCreateReply, ConnectionCreateRequest, ConnectionNodeReference,
        NodeCreateForApplicationRequest, NodeCreateReply, UpdateNodePositionReply,
        UpdateNodePositionRequest,
    },
};
use warp::http::StatusCode;

#[tokio::test]
async fn connection_create() {
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

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let alpha = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;
    let bravo = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/schematicDal/connectionCreate")
        .json(&ConnectionCreateRequest {
            connection: Connection {
                source: ConnectionNodeReference {
                    node_id: alpha.id.clone(),
                    socket_id: "output".to_string(),
                    socket_name: "pants".to_string(),
                    node_kind: "service".to_string(),
                },
                destination: ConnectionNodeReference {
                    node_id: bravo.id.clone(),
                    socket_id: "input".to_string(),
                    socket_name: "poop".to_string(),
                    node_kind: "service".to_string(),
                },
            },
            schematic_kind: si_model::SchematicKind::Component,
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            root_object_id: application.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let _reply: ConnectionCreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize reply");

    let _txn = conn.transaction().await.expect("cannot get transaction");

    // TODO figure out how to check the created edge as the API does not currently return any
    // useful information
}

#[tokio::test]
async fn connection_create_duplicate() {
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

    let txn = conn.transaction().await.expect("cannot get transaction");
    let nats = nats_conn.transaction();

    let alpha = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;
    let tail_vertex = Vertex::new(&alpha.id, &alpha.object_id, "output", "service");
    let bravo = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "service",
    )
    .await;
    let head_vertex = Vertex::new(&bravo.id, &bravo.object_id, "input", "service");

    let _edge = Edge::new(
        &txn,
        &nats,
        tail_vertex,
        head_vertex,
        false,
        EdgeKind::Configures,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create edge");

    txn.commit().await.expect("cannot commit txn");
    nats.commit().await.expect("cannot commit nats txn");

    let token = login_user(&ctx, &nba).await;
    let filter = api(pg, nats_conn, veritech, event_log_fs, secret_key);

    let res = warp::test::request()
        .method("POST")
        .header("authorization", &token)
        .path("/schematicDal/connectionCreate")
        .json(&ConnectionCreateRequest {
            connection: Connection {
                source: ConnectionNodeReference {
                    node_id: alpha.id.clone(),
                    socket_id: "output".to_string(),
                    socket_name: "output".to_string(),
                    node_kind: "service".to_string(),
                },
                destination: ConnectionNodeReference {
                    node_id: bravo.id.clone(),
                    socket_id: "input".to_string(),
                    socket_name: "input".to_string(),
                    node_kind: "service".to_string(),
                },
            },
            schematic_kind: SchematicKind::Component,
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            root_object_id: application.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "edge should not be created"
    );
}

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
        .path("/schematicDal/nodeCreateForApplication")
        .json(&NodeCreateForApplicationRequest {
            name: Some(name.clone()),
            entity_type: "service".to_string(),
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            application_id: application.id.clone(),
            schematic_kind: si_model::SchematicKind::Deployment,
            deployment_selected_entity_id: None,
        })
        .reply(&filter)
        .await;

    assert_eq!(res.status(), StatusCode::OK, "create should succeed");
    let reply: NodeCreateReply =
        serde_json::from_slice(res.body()).expect("cannot deserialize node reply");

    let txn = conn.transaction().await.expect("cannot get transaction");

    let expected_node = Node::get(&txn, &reply.node.node.id)
        .await
        .expect("cannot get node");
    assert_eq!(expected_node.id, reply.node.node.id);
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
        .path("/schematicDal/updateNodePosition")
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
        .path("/schematicDal/updateNodePosition")
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
