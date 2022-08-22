use crate::dal::test;
use dal::DalContext;

use dal::{
    socket::SocketEdgeKind, test::helpers::create_system, Component, Connection, NodePosition,
    Schema, Schematic, StandardModel,
};

#[test]
async fn get_schematic(ctx: &DalContext<'_, '_>) {
    let service_schema = Schema::find_by_attr(ctx, "name", &"service".to_string())
        .await
        .expect("cannot find service schema")
        .pop()
        .expect("no service schema found");

    let service_schema_variant = service_schema
        .default_variant(ctx)
        .await
        .expect("cannot get default schema variant");

    let sockets = service_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let input_socket = sockets
        .iter()
        .find(|s| s.edge_kind() == &SocketEdgeKind::Configures && s.name() == "service")
        .expect("cannot find input socket");
    let explicit_internal_provider = input_socket
        .internal_provider(ctx)
        .await
        .expect("cannot find external provider")
        .expect("external provider not found");

    let output_socket = sockets
        .iter()
        .find(|s| s.edge_kind() == &SocketEdgeKind::Output && s.name() == "service")
        .expect("cannot find output socket");
    let external_provider = output_socket
        .external_provider(ctx)
        .await
        .expect("cannot find external provider")
        .expect("external provider not found");

    let (_component, node) =
        Component::new_for_schema_with_node(ctx, "sc-component-get_schematic", service_schema.id())
            .await
            .expect("unable to create component for schema");

    let system = create_system(ctx).await;

    let node_position = NodePosition::upsert_by_node_id(
        ctx,
        (*service_schema.kind()).into(),
        Some(*system.id()),
        None,
        *node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    let schematic = Schematic::find(ctx, Some(*system.id()))
        .await
        .expect("cannot find schematic");
    assert_eq!(schematic.nodes().len(), 2);
    assert_eq!(schematic.nodes()[1].id(), node.id());
    assert_eq!(
        schematic.nodes()[1].positions()[0].x.to_string(),
        node_position.x()
    );
    assert_eq!(
        schematic.nodes()[1].positions()[0].y.to_string(),
        node_position.y()
    );
    assert_eq!(schematic.connections().len(), 1);
}

#[test]
async fn create_connection(ctx: &DalContext<'_, '_>) {
    let service_schema = Schema::find_by_attr(ctx, "name", &"service".to_string())
        .await
        .expect("cannot find service schema")
        .pop()
        .expect("no service schema found");

    let service_schema_variant = service_schema
        .default_variant(ctx)
        .await
        .expect("cannot get default schema variant");

    let sockets = service_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let input_socket = sockets
        .iter()
        .find(|s| s.edge_kind() == &SocketEdgeKind::Configures && s.name() == "service")
        .expect("cannot find input socket");
    let explicit_internal_provider = input_socket
        .internal_provider(ctx)
        .await
        .expect("cannot find external provider")
        .expect("external provider not found");

    let output_socket = sockets
        .iter()
        .find(|s| s.edge_kind() == &SocketEdgeKind::Output && s.name() == "service")
        .expect("cannot find output socket");
    let external_provider = output_socket
        .external_provider(ctx)
        .await
        .expect("cannot find external provider")
        .expect("external provider not found");

    let (_head_component, head_node) =
        Component::new_for_schema_with_node(ctx, "head", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let (_tail_component, tail_node) =
        Component::new_for_schema_with_node(ctx, "tail", service_schema.id())
            .await
            .expect("cannot create component and node for service");

    let connection = Connection::new(
        ctx,
        head_node.id(),
        output_socket.id(),
        *explicit_internal_provider.id(),
        tail_node.id(),
        input_socket.id(),
        *external_provider.id(),
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
