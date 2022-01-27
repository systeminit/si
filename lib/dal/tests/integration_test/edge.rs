use crate::test_setup;

use dal::edge::{EdgeKind, VertexObjectKind};
use dal::test_harness::{
    create_change_set, create_edit_session, create_system, create_visibility_edit_session,
};
use dal::{
    Component, Edge, HistoryActor, Node, NodeKind, Schema, StandardModel, Tenancy, Visibility,
};

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

#[tokio::test]
async fn include_component_in_system() {
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

    let system_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"system".to_string())
            .await
            .expect("cannot find system schema")
            .pop()
            .expect("no system schema found");
    let system_schema_variant = system_schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot get default schema variant");

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");

    let _service_schema_variant = service_schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot get default schema variant");

    let system = create_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    system
        .set_schema(&txn, &nats, &visibility, &history_actor, system_schema.id())
        .await
        .expect("cannot set schema for system");
    system
        .set_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            system_schema_variant.id(),
        )
        .await
        .expect("cannot set schema variant for system");

    let system_node = Node::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::System,
    )
    .await
    .expect("cannot create node for system");
    system_node
        .set_system(&txn, &nats, &visibility, &history_actor, system.id())
        .await
        .expect("cannot assign system to node");

    let (first_component, first_component_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        "first",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (_second_component, _second_component_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        "second",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let edge = Edge::include_component_in_system(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        first_component.id(),
        system.id(),
    )
    .await
    .expect("cannot create new edge");

    assert_eq!(edge.head_node_id(), *first_component_node.id(),);
}

#[tokio::test]
async fn include_component_in_system_with_edit_sessions() {
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
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);

    let system_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"system".to_string())
            .await
            .expect("cannot find system schema")
            .pop()
            .expect("no system schema found");
    let system_schema_variant = system_schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot get default schema variant");

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");

    let _service_schema_variant = service_schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot get default schema variant");

    let system = create_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    system
        .set_schema(&txn, &nats, &visibility, &history_actor, system_schema.id())
        .await
        .expect("cannot set schema for system");
    system
        .set_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            system_schema_variant.id(),
        )
        .await
        .expect("cannot set schema variant for system");

    let system_node = Node::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &NodeKind::System,
    )
    .await
    .expect("cannot create node for system");
    system_node
        .set_system(&txn, &nats, &visibility, &history_actor, system.id())
        .await
        .expect("cannot assign system to node");

    let (first_component, first_component_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        "first",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (_second_component, _second_component_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        "second",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let edge = Edge::include_component_in_system(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        first_component.id(),
        system.id(),
    )
    .await
    .expect("cannot create new edge");

    assert_eq!(edge.head_node_id(), *first_component_node.id(),);
}
