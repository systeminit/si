use si_model_test::{
    create_edge, create_edge_graph, create_vertex, one_time_setup, signup_new_billing_account,
    TestContext,
};

use si_model::{Edge, EdgeKind};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let head_vertex = create_vertex(&txn, &nats).await;
    let tail_vertex = create_vertex(&txn, &nats).await;
    let edge = Edge::new(
        &txn,
        &nats,
        tail_vertex.clone(),
        head_vertex.clone(),
        false,
        EdgeKind::Includes,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create new edge");

    assert_eq!(&edge.tail_vertex, &tail_vertex);
    assert_eq!(&edge.head_vertex, &head_vertex);
    assert_eq!(&edge.bidirectional, &false);
    assert_eq!(&edge.kind, &EdgeKind::Includes);
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_edge = create_edge(&txn, &nats, &nba).await;

    let same_edge = Edge::get(&txn, &og_edge.id)
        .await
        .expect("cannot get same edge");
    assert_eq!(&og_edge, &same_edge);
}

#[tokio::test]
async fn delete() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let og_edge = create_edge(&txn, &nats, &nba).await;

    let mut same_edge = Edge::get(&txn, &og_edge.id)
        .await
        .expect("cannot get same edge");
    assert_eq!(&og_edge, &same_edge);

    same_edge
        .delete(&txn, &nats)
        .await
        .expect("cannot delete edge");
    let delete_edge_result = Edge::get(&txn, &og_edge.id).await;
    assert!(
        delete_edge_result.is_err(),
        "edge exists and should be deleted"
    );
}

#[tokio::test]
async fn direct_successor_edges_by_node_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::direct_successor_edges_by_node_id(
        &txn,
        &edges.first.kind,
        &edges.first.head_vertex.node_id,
    )
    .await
    .expect("cannot get direct successor edges by node id");

    assert_eq!(results.len(), 2);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(false, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(true, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn all_successor_edges_by_node_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::all_successor_edges_by_node_id(
        &txn,
        &edges.first.kind,
        &edges.first.head_vertex.node_id,
    )
    .await
    .expect("cannot get all successor edges by node id");

    assert_eq!(results.len(), 4);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(true, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(true, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn direct_successor_edges_by_object_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::direct_successor_edges_by_object_id(
        &txn,
        &edges.first.kind,
        &edges.first.head_vertex.object_id,
    )
    .await
    .expect("cannot get all successor edges by object id");

    assert_eq!(results.len(), 2);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(false, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(true, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn all_successor_edges_by_object_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::all_successor_edges_by_object_id(
        &txn,
        &edges.first.kind,
        &edges.first.head_vertex.object_id,
    )
    .await
    .expect("cannot get all successor edges by object id");

    assert_eq!(results.len(), 4);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(true, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(true, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn direct_predecessor_edges_by_node_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::direct_predecessor_edges_by_node_id(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.node_id,
    )
    .await
    .expect("cannot get direct predecessor edges by node id");

    assert_eq!(results.len(), 1);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(false, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn all_predecessor_edges_by_node_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::all_predecessor_edges_by_node_id(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.node_id,
    )
    .await
    .expect("cannot get all predecessor edges by node id");

    assert_eq!(results.len(), 3);

    assert_eq!(true, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn direct_predecessor_edges_by_object_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::direct_predecessor_edges_by_object_id(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.object_id,
    )
    .await
    .expect("cannot get direct predecessor edges by object id");

    assert_eq!(results.len(), 1);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(false, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn all_predecessor_edges_by_object_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::all_predecessor_edges_by_object_id(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.object_id,
    )
    .await
    .expect("cannot get all predecessor edges by object id");

    assert_eq!(results.len(), 3);

    assert_eq!(true, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn by_kind_and_head_node_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results =
        Edge::by_kind_and_head_node_id(&txn, &edges.fourth.kind, &edges.fourth.tail_vertex.node_id)
            .await
            .expect("cannot get results");

    assert_eq!(results.len(), 1);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(false, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn by_kind_and_head_object_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::by_kind_and_head_object_id(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.object_id,
    )
    .await
    .expect("cannot get results");

    assert_eq!(results.len(), 1);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(false, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn by_kind_and_tail_node_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results =
        Edge::by_kind_and_tail_node_id(&txn, &edges.first.kind, &edges.first.head_vertex.node_id)
            .await
            .expect("cannot get results");

    assert_eq!(results.len(), 2);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(false, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(true, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn by_kind_and_tail_object_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::by_kind_and_tail_object_id(
        &txn,
        &edges.first.kind,
        &edges.first.head_vertex.object_id,
    )
    .await
    .expect("cannot get all successor edges by object id");

    assert_eq!(results.len(), 2);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(true, results.iter().any(|e| e == &edges.second));
    assert_eq!(false, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(true, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}

#[tokio::test]
async fn by_kind_and_head_object_id_and_tail_object_type() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let edges = create_edge_graph(&txn, &nats, &nba).await;

    let results = Edge::by_kind_and_head_object_id_and_tail_object_type(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.object_id,
        &edges.fourth.head_vertex.object_type,
    )
    .await
    .expect("cannot get direct predecessor edges by object id");

    assert_eq!(results.len(), 1);

    assert_eq!(false, results.iter().any(|e| e == &edges.first));
    assert_eq!(false, results.iter().any(|e| e == &edges.second));
    assert_eq!(true, results.iter().any(|e| e == &edges.third));
    assert_eq!(false, results.iter().any(|e| e == &edges.fourth));
    assert_eq!(false, results.iter().any(|e| e == &edges.fifth));
    assert_eq!(false, results.iter().any(|e| e == &edges.rogue));
}
