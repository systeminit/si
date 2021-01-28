use crate::models::billing_account::{signup_new_billing_account, NewBillingAccount};
use crate::{one_time_setup, TestContext};
use names::{Generator, Name};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{Edge, EdgeKind, Vertex};

pub async fn create_vertex(_txn: &PgTxn<'_>, _nats: &NatsTxn) -> Vertex {
    // TODO: This should become real nodes/entities and objects!
    let fake_node_id = Generator::with_naming(Name::Numbered).next().unwrap();
    let fake_object_id = Generator::with_naming(Name::Numbered).next().unwrap();
    Vertex::new(
        format!("node:{}", fake_node_id),
        format!("object:{}", fake_object_id),
        "top",
        "soundgarden",
    )
}

pub async fn create_edge(txn: &PgTxn<'_>, nats: &NatsTxn, nba: &NewBillingAccount) -> Edge {
    let head_vertex = create_vertex(&txn, &nats).await;
    let tail_vertex = create_vertex(&txn, &nats).await;
    Edge::new(
        &txn,
        &nats,
        tail_vertex.clone(),
        head_vertex.clone(),
        false,
        EdgeKind::Includes,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create new edge")
}

pub async fn create_edge_with_tail_vertex(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    tail_vertex: &Vertex,
) -> Edge {
    let head_vertex = create_vertex(&txn, &nats).await;
    Edge::new(
        &txn,
        &nats,
        tail_vertex.clone(),
        head_vertex.clone(),
        false,
        EdgeKind::Includes,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create new edge")
}

struct EdgeGraph {
    rogue: Edge,
    first: Edge,
    second: Edge,
    third: Edge,
    fourth: Edge,
    fifth: Edge,
}

async fn create_edge_graph(txn: &PgTxn<'_>, nats: &NatsTxn, nba: &NewBillingAccount) -> EdgeGraph {
    //                        first_edge_tail
    //                             |              \
    //                second_edge_head      fifth_edge_head
    //                             |
    //                          third_edge_head
    //                             |
    //                          fourth_edge_head

    let rogue = create_edge(&txn, &nats, &nba).await;
    let first = create_edge(&txn, &nats, &nba).await;
    let second = create_edge_with_tail_vertex(&txn, &nats, &nba, &first.head_vertex).await;
    let third = create_edge_with_tail_vertex(&txn, &nats, &nba, &second.head_vertex).await;
    let fourth = create_edge_with_tail_vertex(&txn, &nats, &nba, &third.head_vertex).await;
    let fifth = create_edge_with_tail_vertex(&txn, &nats, &nba, &first.head_vertex).await;

    EdgeGraph {
        rogue,
        first,
        second,
        third,
        fourth,
        fifth,
    }
}

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
async fn list() {
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

    let _primary_edge = create_edge(&txn, &nats, &nba).await;
    let _secondary_edge = create_edge(&txn, &nats, &nba).await;
    let _tertiary_edge = create_edge(&txn, &nats, &nba).await;
    let reply = Edge::list(&txn, &nba.billing_account.id, None, None, None, None, None)
        .await
        .expect("cannot list edges");
    assert_eq!(reply.items.len(), 3);
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
async fn by_kind_and_head_object_id_and_tail_type_name() {
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

    let results = Edge::by_kind_and_head_object_id_and_tail_type_name(
        &txn,
        &edges.fourth.kind,
        &edges.fourth.tail_vertex.object_id,
        &edges.fourth.head_vertex.type_name,
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
