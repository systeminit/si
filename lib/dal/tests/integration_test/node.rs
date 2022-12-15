use dal::{
    node::{NodeKind, NodeTemplate},
    DalContext, HistoryActor, Node, SchemaKind, StandardModel, Visibility, WriteTenancy,
};
use dal_test::{
    test,
    test_harness::{
        create_component_and_schema_with_variant, create_node, create_schema, create_schema_variant,
    },
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
    let component = create_component_and_schema_with_variant(ctx).await;
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

#[test]
async fn new_node_template(ctx: &DalContext) {
    let mut schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let node_template = NodeTemplate::new_from_schema_id(ctx, *schema.id())
        .await
        .expect("cannot create node template");
    assert_eq!(node_template.title, schema.name());
}
