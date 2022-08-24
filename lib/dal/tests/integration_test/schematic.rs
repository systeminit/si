use crate::dal::test;

use dal::socket::{SocketArity, SocketKind};
use dal::test::helpers::builtins::BuiltinsHarness;
use dal::{
    socket::SocketEdgeKind, Connection, DalContext, NodePosition, Schema, Schematic, SchematicKind,
    StandardModel,
};
use dal::{SchemaVariant, SchematicEdgeView};

#[test]
async fn get_schematic_and_create_connection(ctx: &DalContext<'_, '_>) {
    let mut harness = BuiltinsHarness::new();
    let tail_docker_hub_credential = harness.create_docker_hub_credential(ctx, "tail").await;
    let head_docker_image = harness.create_docker_image(ctx, "head").await;

    let tail_schema = Schema::get_by_id(ctx, &tail_docker_hub_credential.schema_id)
        .await
        .expect("could not find schema by id")
        .expect("schema by id not found");
    let head_schema = Schema::get_by_id(ctx, &head_docker_image.schema_id)
        .await
        .expect("could not find schema by id")
        .expect("schema by id not found");

    let tail_schema_variant =
        SchemaVariant::get_by_id(ctx, &tail_docker_hub_credential.schema_variant_id)
            .await
            .expect("could not find schema variant by id")
            .expect("schema variant by id not found");
    let head_schema_variant = SchemaVariant::get_by_id(ctx, &head_docker_image.schema_variant_id)
        .await
        .expect("could not find schema variant by id")
        .expect("schema variant by id not found");

    let tail_sockets = tail_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");
    let head_sockets = head_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let output_socket = tail_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::Output
                && s.kind() == &SocketKind::Provider
                && s.schematic_kind() == &SchematicKind::Component
                && s.arity() == &SocketArity::Many
                && s.name() == "docker_hub_credential"
        })
        .expect("cannot find output socket");
    let external_provider = output_socket
        .external_provider(ctx)
        .await
        .expect("cannot find external provider")
        .expect("external provider not found");

    let input_socket = head_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::Configures
                && s.kind() == &SocketKind::Provider
                && s.schematic_kind() == &SchematicKind::Component
                && s.arity() == &SocketArity::Many
                && s.name() == "docker_hub_credential"
        })
        .expect("cannot find input socket");
    let explicit_internal_provider = input_socket
        .internal_provider(ctx)
        .await
        .expect("cannot find external provider")
        .expect("external provider not found");

    let tail_node_position = NodePosition::upsert_by_node_id(
        ctx,
        (*tail_schema.kind()).into(),
        None,
        None,
        *tail_docker_hub_credential.node.id(),
        "123",
        "-10",
    )
    .await
    .expect("cannot upsert node position");

    let head_node_position = NodePosition::upsert_by_node_id(
        ctx,
        (*head_schema.kind()).into(),
        None,
        None,
        *head_docker_image.node.id(),
        "124",
        "-11",
    )
    .await
    .expect("cannot upsert node position");

    let connection = Connection::new(
        ctx,
        head_docker_image.node.id(),
        input_socket.id(),
        *explicit_internal_provider.id(),
        tail_docker_hub_credential.node.id(),
        output_socket.id(),
        *external_provider.id(),
    )
    .await
    .expect("could not create connection");

    let schematic = Schematic::find(ctx, None)
        .await
        .expect("cannot find schematic");

    // Check the nodes.
    assert_eq!(schematic.nodes().len(), 2);
    let tail_node_id = *tail_docker_hub_credential.node.id();
    let tail_node_id: i64 = tail_node_id.into();
    assert_eq!(schematic.nodes()[0].id(), &tail_node_id.to_string());
    let head_node_id = *head_docker_image.node.id();
    let head_node_id: i64 = head_node_id.into();
    assert_eq!(schematic.nodes()[1].id(), &head_node_id.to_string(),);

    // Check the node positions.
    assert_eq!(
        schematic.nodes()[0].position().x().to_string(),
        tail_node_position.x()
    );
    assert_eq!(
        schematic.nodes()[0].position().y().to_string(),
        tail_node_position.y()
    );
    assert_eq!(
        schematic.nodes()[1].position().x().to_string(),
        head_node_position.x()
    );
    assert_eq!(
        schematic.nodes()[1].position().y().to_string(),
        head_node_position.y()
    );

    // Check the connection on the schematic.
    assert_eq!(schematic.edges().len(), 1);
    assert_eq!(
        schematic.edges()[0],
        SchematicEdgeView::from(connection.clone())
    );

    // Check the connection itself.
    let (source_node_id, source_socket_id) = connection.source();
    let (destination_node_id, destination_socket_id) = connection.destination();

    assert_eq!(source_node_id, *tail_docker_hub_credential.node.id());
    assert_eq!(source_socket_id, *output_socket.id());
    assert_eq!(destination_node_id, *head_docker_image.node.id());
    assert_eq!(destination_socket_id, *input_socket.id());
}
