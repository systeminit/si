use dal::{
    node::NodeKind, DalContext, HistoryActor, Node, StandardModel, Visibility, WriteTenancy,
};
use dal_test::{
    test,
    test_harness::{create_component_and_schema, create_node},
};

#[test]
async fn new(ctx: &DalContext) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let _node = Node::new(ctx, &NodeKind::Configuration)
        .await
        .expect("cannot create node");
}

#[test]
async fn component_relationships(ctx: &DalContext) {
    let component = create_component_and_schema(ctx).await;
    let node = create_node(ctx, &NodeKind::Configuration).await;
    node.set_component(ctx, component.id())
        .await
        .expect("cannot associate node with component");
    let retrieved_component = node
        .component(ctx)
        .await
        .expect("cannot retrieve component for node")
        .expect("no component set for node");
    assert_eq!(&retrieved_component, &component);
}
