use crate::test_setup;

use dal::node::NodeKind;
use dal::test_harness::{create_component_and_schema, create_node};
use dal::{HistoryActor, Node, StandardModel, Tenancy, Visibility};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _node = Node::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Component,
    )
    .await
    .expect("cannot create node");
}

#[tokio::test]
async fn component_relationships() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let component =
        create_component_and_schema(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let node = create_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Component,
    )
    .await;
    node.set_component(&txn, &nats, &visibility, &history_actor, component.id())
        .await
        .expect("cannot associate node with component");
    let retrieved_component = node
        .component(&txn, &visibility)
        .await
        .expect("cannot retrieve component for node")
        .expect("no component set for node");
    assert_eq!(&retrieved_component, &component);
}
