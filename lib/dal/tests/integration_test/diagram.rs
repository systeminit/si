use dal::node_position::NodePositionView;
use dal::{
    socket::{SocketArity, SocketEdgeKind, SocketKind},
    Component, ComponentView, Connection, DalContext, Diagram, DiagramEdgeView, DiagramKind,
    NodePosition, NodeTemplate, NodeView, Schema, SchemaVariant, StandardModel,
};
use dal_test::helpers::component_view::ComponentViewProperties;
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_node_and_check_intra_component_intelligence(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("could not perform schema find by attr")
        .pop()
        .expect("docker image schema not found");
    let schema_variant_id = *schema
        .default_schema_variant_id()
        .expect("could not find default schema variant id");
    let name = "13700KF".to_string();

    let (component, node) = Component::new(ctx, &name, schema_variant_id)
        .await
        .expect("could not create component");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not get component view");
    let mut component_view_properties = ComponentViewProperties::try_from(component_view)
        .expect("could not create component view properties from component view");
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "13700KF"
            },
            "si": {
                "name": "13700KF",
                "type": "component"
            },
        }], // expected
        component_view_properties.drop_qualification().to_value() // actual
    );

    let node_template = NodeTemplate::new_from_schema_id(ctx, *schema.id())
        .await
        .expect("could not create node template");
    let diagram_kind = schema.diagram_kind().expect("no diagram kind for schema");
    assert_eq!(diagram_kind, DiagramKind::Configuration);

    let position = NodePosition::new(
        ctx,
        *node.id(),
        diagram_kind,
        "0",
        "0",
        Some("500"),
        Some("500"),
    )
    .await
    .expect("could not create node position");
    let positions = vec![NodePositionView::from(position)];
    let node_view = NodeView::new(name, &node, *component.id(), positions, node_template);

    let found_component = Component::get_by_id(ctx, &node_view.component_id())
        .await
        .expect("could not perform get by id for component")
        .expect("component not found by id");
    assert_eq!(*component.id(), *found_component.id());

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not get component view");
    let mut component_view_properties = ComponentViewProperties::try_from(component_view)
        .expect("could not create component view properties from component view");
    assert_eq!(
        serde_json::json![{
            "domain": {
                "image": "13700KF"
            },
            "si": {
                "name": "13700KF",
                "type": "component"
            },
        }], // expected
        component_view_properties.drop_qualification().to_value() // actual
    );
}

#[test]
async fn get_diagram_and_create_connection(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let from_docker_hub_credential = harness
        .create_component(ctx, "from", Builtin::DockerHubCredential)
        .await;
    let to_docker_image = harness
        .create_component(ctx, "to", Builtin::DockerImage)
        .await;

    let _from_schema = Schema::get_by_id(ctx, &from_docker_hub_credential.schema_id)
        .await
        .expect("could not find schema by id")
        .expect("schema by id not found");
    let _to_schema = Schema::get_by_id(ctx, &to_docker_image.schema_id)
        .await
        .expect("could not find schema by id")
        .expect("schema by id not found");

    let from_schema_variant =
        SchemaVariant::get_by_id(ctx, &from_docker_hub_credential.schema_variant_id)
            .await
            .expect("could not find schema variant by id")
            .expect("schema variant by id not found");
    let to_schema_variant = SchemaVariant::get_by_id(ctx, &to_docker_image.schema_variant_id)
        .await
        .expect("could not find schema variant by id")
        .expect("schema variant by id not found");

    let from_sockets = from_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");
    let to_sockets = to_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let output_socket = from_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationOutput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Docker Hub Credential"
        })
        .expect("cannot find output socket");

    let input_socket = to_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationInput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Docker Hub Credential"
        })
        .expect("cannot find input socket");

    let from_node_position = NodePosition::upsert_by_node_id(
        ctx,
        DiagramKind::Configuration,
        from_docker_hub_credential.node_id,
        "123",
        "-10",
        Some("500"),
        Some("500"),
    )
    .await
    .expect("cannot upsert node position");

    let to_node_position = NodePosition::upsert_by_node_id(
        ctx,
        DiagramKind::Configuration,
        to_docker_image.node_id,
        "124",
        "-11",
        Some("500"),
        Some("500"),
    )
    .await
    .expect("cannot upsert node position");

    let connection = Connection::new(
        ctx,
        from_docker_hub_credential.node_id,
        *output_socket.id(),
        to_docker_image.node_id,
        *input_socket.id(),
    )
    .await
    .expect("could not create connection");

    let diagram = Diagram::assemble(ctx).await.expect("cannot find diagram");

    // Check the nodes.
    assert_eq!(diagram.nodes().len(), 2);
    assert_eq!(
        diagram
            .nodes()
            .iter()
            .filter(|n| n.id() == from_docker_hub_credential.node_id.to_string()
                || n.id() == to_docker_image.node_id.to_string())
            .count(),
        2
    );

    // Check the node positions.
    assert_eq!(
        diagram
            .nodes()
            .iter()
            .filter(|n| (n.position().x().to_string() == from_node_position.x()
                && n.position().y().to_string() == from_node_position.y())
                || (n.position().x().to_string() == to_node_position.x()
                    && n.position().y().to_string() == to_node_position.y()))
            .count(),
        2
    );

    // Check the connection on the diagram.
    assert_eq!(diagram.edges().len(), 1);
    assert_eq!(
        diagram.edges()[0],
        DiagramEdgeView::from(connection.clone())
    );

    // Check the connection itself.
    let (source_node_id, source_socket_id) = connection.source();
    let (destination_node_id, destination_socket_id) = connection.destination();

    assert_eq!(source_node_id, from_docker_hub_credential.node_id);
    assert_eq!(source_socket_id, *output_socket.id());
    assert_eq!(destination_node_id, to_docker_image.node_id);
    assert_eq!(destination_socket_id, *input_socket.id());
}
