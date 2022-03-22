use crate::dal::test;
use crate::test_setup;
use dal::{
    test_harness::find_or_create_production_system, Component, Connection, HistoryActor,
    NodePosition, Schema, Schematic, SchematicKind, StandardModel, SystemId, Tenancy, Visibility,
};

#[test]
async fn get_schematic() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let application_schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"application".to_string(),
    )
    .await
    .expect("cannot find application schema")
    .pop()
    .expect("no application schema found");
    let (_component, root_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "sc-component-root-get_schematic",
        application_schema.id(),
    )
    .await
    .expect("unable to create component for schema");

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");
    let (_component, node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "sc-component-get_schematic",
        service_schema.id(),
    )
    .await
    .expect("unable to create component for schema");

    let node_position = NodePosition::upsert_by_node_id(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        SchematicKind::Component,
        &Some(SystemId::from(1)),
        *root_node.id(),
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    let schematic = Schematic::find(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        Some(SystemId::from(1)),
        *root_node.id(),
    )
    .await
    .expect("cannot find schematic");
    assert_eq!(schematic.nodes()[0].id(), root_node.id());
    assert_eq!(schematic.nodes()[1].id(), node.id());
    assert_eq!(schematic.nodes()[1].positions()[0].x(), node_position.x());
    assert_eq!(schematic.nodes()[1].positions()[0].y(), node_position.y());
}

#[test]
async fn create_connection() {
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
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let service_schema =
        Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &"service".to_string())
            .await
            .expect("cannot find service schema")
            .pop()
            .expect("no service schema found");

    let service_schema_variant = service_schema
        .default_variant(
            &txn,
            &tenancy
                .clone_into_read_tenancy(&txn)
                .await
                .expect("unable to generate read tenancy"),
            &visibility,
        )
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

    let (_head_component, head_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "head",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let (_tail_component, tail_node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "tail",
        service_schema.id(),
    )
    .await
    .expect("cannot create component and node for service");

    let connection = Connection::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        head_node.id(),
        output_socket.id(),
        tail_node.id(),
        input_socket.id(),
    )
    .await
    .expect("could not create connection");

    let (source_node_id, source_socket_id) = connection.source();
    let (destination_node_id, destination_socket_id) = connection.destination();

    assert_eq!(source_node_id, *head_node.id());
    assert_eq!(source_socket_id, output_socket.id().to_owned());
    assert_eq!(destination_node_id, *tail_node.id());
    assert_eq!(destination_socket_id, input_socket.id().to_owned());
}
