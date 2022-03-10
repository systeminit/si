use crate::test_setup;

use dal::node::{NodeKind, NodeTemplate};
use dal::test_harness::{
    create_component_and_schema, create_node, create_schema, create_schema_variant,
};
use dal::{HistoryActor, Node, SchemaKind, StandardModel, Tenancy, Visibility};
use test_env_log::test;

#[test(tokio::test)]
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

#[test(tokio::test)]
async fn component_relationships() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let component = create_component_and_schema(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
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

#[test(tokio::test)]
async fn new_node_template() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concept,
    )
    .await;
    let schema_variant = create_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech,
        encr_key,
    )
    .await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema for variant");
    schema
        .set_default_schema_variant_id(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let node_template = NodeTemplate::new_from_schema_id(&txn, &tenancy, &visibility, *schema.id())
        .await
        .expect("cannot create node template");
    assert_eq!(node_template.label.title, schema.name());
}
