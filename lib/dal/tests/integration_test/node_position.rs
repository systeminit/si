use dal::{node::NodeKind, DalContext, DiagramKind, NodePosition, StandardModel};
use dal_test::{test, test_harness::create_node};

#[test]
async fn new(ctx: &DalContext) {
    let node = create_node(ctx, &NodeKind::Configuration).await;
    let node_position = NodePosition::new(
        ctx,
        *node.id(),
        DiagramKind::Configuration,
        None,
        "123",
        "-10",
    )
    .await
    .expect("cannot create node position");

    assert_eq!(node_position.x(), "123");
    assert_eq!(node_position.y(), "-10");
    assert_eq!(
        NodePosition::get_by_pk(ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .node(ctx)
            .await
            .expect("failed to get the node for the node position")
            .expect("node not set")
            .id(),
        node.id()
    );
}

#[test]
async fn set_node_position(ctx: &DalContext) {
    let node = create_node(ctx, &NodeKind::Configuration).await;

    let node_position = NodePosition::upsert_by_node_id(
        ctx,
        DiagramKind::Configuration,
        None,
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::get_by_pk(ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .x(),
        "123"
    );
    assert_eq!(
        NodePosition::get_by_pk(ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .y(),
        "-10"
    );

    let node_position = NodePosition::upsert_by_node_id(
        ctx,
        DiagramKind::Configuration,
        None,
        *node.id(),
        "-10",
        "123",
    )
    .await
    .expect("cannot upsert node position");

    assert_eq!(
        NodePosition::get_by_pk(ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .x(),
        "-10"
    );
    assert_eq!(
        NodePosition::get_by_pk(ctx, node_position.pk())
            .await
            .expect("failed to get node position by pk")
            .y(),
        "123"
    );
}
