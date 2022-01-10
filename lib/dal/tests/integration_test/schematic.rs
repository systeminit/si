use dal::{
    node::NodeKind, test_harness::create_node, HistoryActor, NodePosition, SchematicKind,
    StandardModel, Tenancy, Visibility,
};

use crate::test_setup;

#[tokio::test]
async fn get_schematic() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let root_node = create_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Component,
    )
    .await;
    let node_position = NodePosition::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        SchematicKind::Component,
        *root_node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot create node position");

    let schematic = Schematic::find(&txn, &tenancy, &visibility, None, *root_node.id()).await.expect("cannot find schematic");
    assert_eq!(schematic.nodes.first().expect("no node found on schematic").id() == node.id());
}
