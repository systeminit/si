use dal::{node::NodeKind, DalContext, Node, StandardModel};
use dal_test::{
    test,
    test_harness::{create_component_and_schema, create_node},
};

#[test]
async fn new(ctx: &DalContext) {
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
