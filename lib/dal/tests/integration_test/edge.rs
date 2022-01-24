use crate::test_setup;

use dal::edge::{EdgeKind, VertexObjectKind};
use dal::{Component, Edge, HistoryActor, Schema, StandardModel, Tenancy, Visibility};

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");

    let service_schema_variant = service_schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot get default schema variant");

    let sockets = service_schema_variant
        .sockets(&txn, &visibility)
        .await
        .expect("cannot fetch sockets");

    let input_socket = sockets
        .iter()
        .find(|s| s.name() == "input")
        .expect("cannot find input socket");

    let output_socket = sockets
        .iter()
        .find(|s| s.name() == "output")
        .expect("cannot find output socket");

    let (head_component, head_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        "head",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (tail_component, tail_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        "head",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let _edge = Edge::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        EdgeKind::Configures,
        *head_node.id(),
        VertexObjectKind::Component,
        (*head_component.id()).into(),
        *input_socket.id(),
        *tail_node.id(),
        VertexObjectKind::Component,
        (*tail_component.id()).into(),
        *output_socket.id(),
    )
    .await
    .expect("cannot create new edge");
}
