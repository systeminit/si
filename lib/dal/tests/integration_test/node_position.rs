use crate::dal::test;
use dal::{
    node::NodeKind, test_harness::create_node, HistoryActor, NodePosition, SchematicKind,
    StandardModel, Tenancy, Visibility,
};

use crate::test_setup;

#[test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
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
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        *root_node.id(),
        None,
        None,
        "123",
        "-10",
    )
    .await
    .expect("cannot create node position");
    assert_eq!(node_position.x(), "123");
    assert_eq!(node_position.y(), "-10");
}

#[test]
async fn set_node() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
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
    let node = create_node(
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
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        *root_node.id(),
        None,
        None,
        "123",
        "-10",
    )
    .await
    .expect("cannot create node position");

    node_position
        .set_node(&txn, &nats, &visibility, &history_actor, node.id())
        .await
        .expect("cannot associate node position with node");
    assert_eq!(
        NodePosition::get_by_pk(&txn, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .node(&txn, &visibility)
            .await
            .expect("failed to get the node for the node position")
            .expect("node not set")
            .id(),
        node.id()
    );
}

#[test]
async fn set_node_position() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
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
    let node = create_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Component,
    )
    .await;

    let node_position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        None,
        None,
        *root_node.id(),
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::get_by_pk(&txn, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .x(),
        "123"
    );
    assert_eq!(
        NodePosition::get_by_pk(&txn, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .y(),
        "-10"
    );

    let node_position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        None,
        None,
        *root_node.id(),
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::get_by_pk(&txn, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .x(),
        "-10"
    );
    assert_eq!(
        NodePosition::get_by_pk(&txn, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .y(),
        "123"
    );
}

#[test]
async fn multiple_per_node() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let read_tenancy = tenancy
        .clone_into_read_tenancy(&txn)
        .await
        .expect("unable to generate read tenancy");
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let root_node = create_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Deployment,
    )
    .await;
    let node = create_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::Deployment,
    )
    .await;

    let node_position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Deployment,
        None,
        None,
        *root_node.id(),
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");
    let node_position2 = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        None,
        Some(*node.id()),
        *root_node.id(),
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::find_by_node_id(
            &txn,
            &read_tenancy,
            &visibility,
            None,
            *root_node.id(),
            *node.id()
        )
        .await
        .expect("failed to get node position by pk"),
        vec![node_position.clone(), node_position2]
    );

    let node_position2 = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        None,
        Some(*node.id()),
        *root_node.id(),
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::find_by_node_id(
            &txn,
            &read_tenancy,
            &visibility,
            None,
            *root_node.id(),
            *node.id()
        )
        .await
        .expect("failed to get node position by pk"),
        vec![node_position, node_position2.clone()]
    );

    let node_position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Deployment,
        None,
        None,
        *root_node.id(),
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::find_by_node_id(
            &txn,
            &read_tenancy,
            &visibility,
            None,
            *root_node.id(),
            *node.id()
        )
        .await
        .expect("failed to get node position by pk"),
        vec![node_position, node_position2]
    );
}
