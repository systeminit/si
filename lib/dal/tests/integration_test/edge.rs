use crate::test_setup;

use dal::edge::{EdgeKind, VertexObjectKind};
use dal::test_harness::{
    create_change_set, create_edit_session, create_visibility_edit_session,
    find_or_create_production_system,
};
use dal::{Component, Edge, HistoryActor, Schema, StandardModel, System, Tenancy, Visibility};

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

    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

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
    let (_system, system_node) = System::new_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "production",
    )
    .await
    .expect("cannot create production system");

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");

    let (_first_component, first_component_node) = Component::new_for_schema_with_node(
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

    let (_second_component, second_component_node) = Component::new_for_schema_with_node(
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

    let edges = Edge::find_by_attr(&txn, &tenancy, &visibility, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges");

    assert_eq!(edges.len(), 2);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(edges[0].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[0].tail_node_id(), *system_node.id());
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[1].head_node_id(), *second_component_node.id());
    assert_eq!(edges[1].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);
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
    let (_system, system_node) = System::new_with_node(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "production",
    )
    .await
    .expect("cannot create production system");

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");

    let (_first_component, first_component_node) = Component::new_for_schema_with_node(
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

    let (_second_component, second_component_node) = Component::new_for_schema_with_node(
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

    let edges = Edge::find_by_attr(&txn, &tenancy, &visibility, "kind", &"includes".to_string())
        .await
        .expect("cannot retrieve edges from HEAD");
    assert_eq!(edges.len(), 0);

    let edges = Edge::find_by_attr(
        &txn,
        &tenancy,
        &edit_session_visibility,
        "kind",
        &"includes".to_string(),
    )
    .await
    .expect("cannot retrieve edges from edit session");

    assert_eq!(edges.len(), 2);

    assert_eq!(edges[0].head_node_id(), *first_component_node.id());
    assert_eq!(edges[0].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[0].tail_node_id(), *system_node.id());
    assert_eq!(edges[0].tail_object_kind(), &VertexObjectKind::System);

    assert_eq!(edges[1].head_node_id(), *second_component_node.id());
    assert_eq!(edges[1].head_object_kind(), &VertexObjectKind::Component);
    assert_eq!(edges[1].tail_node_id(), *system_node.id());
    assert_eq!(edges[1].tail_object_kind(), &VertexObjectKind::System);
}
