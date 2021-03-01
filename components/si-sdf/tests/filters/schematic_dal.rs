use crate::{
    filters::{application_context_dal::create_application, session_dal::login_user},
    models::{
        billing_account::signup_new_billing_account, change_set::create_change_set,
        edit_session::create_edit_session, node::create_custom_entity_node,
    },
    one_time_setup, TestContext,
};
use si_sdf::{
    filters::api,
    handlers::{
        application_dal::CreateApplicationReply,
        editor_dal::{
            NodeCreateForApplicationRequest, NodeCreateReply, UpdateNodePositionReply,
            UpdateNodePositionRequest,
        },
        schematic_dal::{
            Connection, ConnectionCreateReply, ConnectionCreateRequest, ConnectionNodeReference,
        },
    },
    models::{Edge, Entity, Node, NodeKind, NodePosition, Vertex},
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

    let alpha = create_custom_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &nba.system,
        &change_set,
        &edit_session,
        "service",
    )
    .await;
    let bravo = create_custom_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &nba.system,
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
                kind: "configures".to_string(),
                source: ConnectionNodeReference {
                    node_id: alpha.id.clone(),
                    socket_id: "output".to_string(),
                    node_kind: "service".to_string(),
                },
                destination: ConnectionNodeReference {
                    node_id: bravo.id.clone(),
                    socket_id: "input".to_string(),
                    node_kind: "service".to_string(),
                },
                system_id: nba.system.id.clone(),
            },
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            application_id: application.id.clone(),
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

    let alpha = create_custom_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &nba.system,
        &change_set,
        &edit_session,
        "service",
    )
    .await;
    let tail_vertex = Vertex::new(&alpha.id, &alpha.object_id, "output", "service");
    let bravo = create_custom_entity_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &nba.system,
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
        si_sdf::models::EdgeKind::Configures,
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
                kind: "configures".to_string(),
                source: ConnectionNodeReference {
                    node_id: alpha.id.clone(),
                    socket_id: "output".to_string(),
                    node_kind: "service".to_string(),
                },
                destination: ConnectionNodeReference {
                    node_id: bravo.id.clone(),
                    socket_id: "input".to_string(),
                    node_kind: "service".to_string(),
                },
                system_id: nba.system.id.clone(),
            },
            workspace_id: nba.workspace.id.clone(),
            change_set_id: change_set.id.clone(),
            edit_session_id: edit_session.id.clone(),
            application_id: application.id.clone(),
        })
        .reply(&filter)
        .await;

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "edge should not be created"
    );
}
