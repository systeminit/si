use crate::model::billing_account::NewBillingAccount;
use names::{Generator, Name};

use si_data::{NatsTxn, PgTxn};
use si_model::{Edge, EdgeKind, Vertex};

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

pub struct EdgeGraph {
    pub rogue: Edge,
    pub first: Edge,
    pub second: Edge,
    pub third: Edge,
    pub fourth: Edge,
    pub fifth: Edge,
}

pub async fn create_edge_graph(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> EdgeGraph {
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
