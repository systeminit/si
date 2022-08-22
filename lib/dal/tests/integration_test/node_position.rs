use crate::dal::test;
use dal::{node::NodeKind, test_harness::create_node, NodePosition, SchematicKind, StandardModel};
use dal::{BillingAccountSignup, DalContext};

#[test]
async fn new(ctx: &DalContext<'_, '_>, _nba: &BillingAccountSignup) {
    let node_position = NodePosition::new(&ctx, SchematicKind::Component, None, None, "123", "-10")
        .await
        .expect("cannot create node position");
    assert_eq!(node_position.x(), "123");
    assert_eq!(node_position.y(), "-10");
}

#[test]
async fn set_node(ctx: &DalContext<'_, '_>) {
    let node = create_node(&ctx, &NodeKind::Component).await;
    let node_position = NodePosition::new(&ctx, SchematicKind::Component, None, None, "123", "-10")
        .await
        .expect("cannot create node position");

    node_position
        .set_node(&ctx, node.id())
        .await
        .expect("cannot associate node position with node");
    assert_eq!(
        NodePosition::get_by_pk(&ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .node(&ctx)
            .await
            .expect("failed to get the node for the node position")
            .expect("node not set")
            .id(),
        node.id()
    );
}

#[test]
async fn set_node_position(ctx: &DalContext<'_, '_>) {
    let node = create_node(&ctx, &NodeKind::Component).await;

    let node_position = NodePosition::upsert_by_node_id(
        &ctx,
        SchematicKind::Component,
        None,
        None,
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::get_by_pk(&ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .x(),
        "123"
    );
    assert_eq!(
        NodePosition::get_by_pk(&ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .y(),
        "-10"
    );

    let node_position = NodePosition::upsert_by_node_id(
        &ctx,
        SchematicKind::Component,
        None,
        None,
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::get_by_pk(&ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .x(),
        "-10"
    );
    assert_eq!(
        NodePosition::get_by_pk(&ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .y(),
        "123"
    );
}

#[test]
async fn multiple_per_node(ctx: &DalContext<'_, '_>) {
    let node = create_node(&ctx, &NodeKind::Deployment).await;

    let node_position = NodePosition::upsert_by_node_id(
        &ctx,
        SchematicKind::Deployment,
        None,
        None,
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");
    let node_position2 = NodePosition::upsert_by_node_id(
        &ctx,
        SchematicKind::Component,
        None,
        Some(*node.id()),
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::find_by_node_id(&ctx, None, *node.id())
            .await
            .expect("failed to get node position by pk"),
        vec![node_position.clone(), node_position2]
    );

    let node_position2 = NodePosition::upsert_by_node_id(
        &ctx,
        SchematicKind::Component,
        None,
        Some(*node.id()),
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::find_by_node_id(&ctx, None, *node.id())
            .await
            .expect("failed to get node position by pk"),
        vec![node_position, node_position2.clone()]
    );

    let node_position = NodePosition::upsert_by_node_id(
        &ctx,
        SchematicKind::Deployment,
        None,
        None,
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::find_by_node_id(&ctx, None, *node.id())
            .await
            .expect("failed to get node position by pk"),
        vec![node_position, node_position2]
    );
}
