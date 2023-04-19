use dal::change_status::ChangeStatus;
use dal::edge::EdgeKind;
use dal::{
    socket::{SocketArity, SocketEdgeKind, SocketKind},
    Component, ComponentView, ComponentViewProperties, Connection, DalContext, Diagram,
    DiagramEdgeView, DiagramKind, Node, Schema, SchemaVariant, StandardModel,
};
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
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("could not find default schema variant id");
    let name = "13700KF".to_string();

    let (component, mut node) = Component::new(ctx, &name, *schema_variant_id)
        .await
        .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not get component view");
    let mut component_view_properties = ComponentViewProperties::try_from(component_view)
        .expect("could not create component view properties from component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "13700KF",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "13700KF",
            },
        }], // expected
        component_view_properties
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    node.set_geometry(ctx, "0", "0", Some("500"), Some("500"))
        .await
        .expect("Could not set node geometry");

    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not get component view");
    let mut component_view_properties = ComponentViewProperties::try_from(component_view)
        .expect("could not create component view properties from component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "13700KF",
                "color": "#4695E7",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "image": "13700KF",
            },
        }], // expected
        component_view_properties
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
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

    let mut from_node = Node::get_by_id(ctx, &from_docker_hub_credential.node_id)
        .await
        .expect("Couldn't find node for from_docker_hub_credential")
        .unwrap();

    from_node
        .set_geometry(ctx, "123", "-10", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    let mut to_node = Node::get_by_id(ctx, &to_docker_image.node_id)
        .await
        .expect("Couldn't find node for to_docker_image")
        .unwrap();

    to_node
        .set_geometry(ctx, "124", "-11", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    let connection = Connection::new(
        ctx,
        from_docker_hub_credential.node_id,
        *output_socket.id(),
        to_docker_image.node_id,
        *input_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    let diagram = Diagram::assemble(ctx).await.expect("cannot find diagram");

    // Check the nodes.
    assert_eq!(diagram.components().len(), 2);
    assert_eq!(
        diagram
            .components()
            .iter()
            .filter(|n| n.node_id() == from_docker_hub_credential.node_id
                || n.node_id() == to_docker_image.node_id)
            .count(),
        2
    );

    // Check the node positions.
    assert_eq!(
        diagram
            .components()
            .iter()
            .filter(|n| (n.position().x().to_string() == from_node.x()
                && n.position().y().to_string() == from_node.y())
                || (n.position().x().to_string() == to_node.x()
                    && n.position().y().to_string() == to_node.y()))
            .count(),
        2
    );

    // Check the connection on the diagram.
    assert_eq!(diagram.edges().len(), 1);
    assert_eq!(
        diagram.edges()[0],
        DiagramEdgeView::from_with_change_status(connection.clone(), ChangeStatus::Added)
    );

    // Check the connection itself.
    let (source_node_id, source_socket_id) = connection.source();
    let (destination_node_id, destination_socket_id) = connection.destination();

    assert_eq!(source_node_id, from_docker_hub_credential.node_id);
    assert_eq!(source_socket_id, *output_socket.id());
    assert_eq!(destination_node_id, to_docker_image.node_id);
    assert_eq!(destination_socket_id, *input_socket.id());
}

#[test]
async fn get_diagram_and_delete_connection(ctx: &DalContext) {
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

    Node::get_by_id(ctx, &from_docker_hub_credential.node_id)
        .await
        .expect("Can't find node")
        .unwrap()
        .set_geometry(ctx, "123", "-10", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    Node::get_by_id(ctx, &to_docker_image.node_id)
        .await
        .expect("Can't find node")
        .unwrap()
        .set_geometry(ctx, "124", "-11", Some("500"), Some("500"))
        .await
        .expect("cannot upsert node position");

    let connection = Connection::new(
        ctx,
        from_docker_hub_credential.node_id,
        *output_socket.id(),
        to_docker_image.node_id,
        *input_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    let diagram = Diagram::assemble(ctx).await.expect("cannot find diagram");

    // Check the nodes.
    assert_eq!(diagram.components().len(), 2);

    // Check the connection on the diagram.
    assert_eq!(diagram.edges().len(), 1);
    assert_eq!(
        diagram.edges()[0],
        DiagramEdgeView::from_with_change_status(connection.clone(), ChangeStatus::Added)
    );

    // Check the connection itself.
    let (source_node_id, source_socket_id) = connection.source();
    let (destination_node_id, destination_socket_id) = connection.destination();

    assert_eq!(source_node_id, from_docker_hub_credential.node_id);
    assert_eq!(source_socket_id, *output_socket.id());
    assert_eq!(destination_node_id, to_docker_image.node_id);
    assert_eq!(destination_socket_id, *input_socket.id());

    // Delete the connection
    let _result = Connection::delete_for_edge(ctx, connection.id).await;

    // let's reassemble the diagram to get the update
    let diagram = Diagram::assemble(ctx).await.expect("cannot find diagram");

    // Check the nodes.
    assert_eq!(diagram.components().len(), 2);

    // Check that no connections exist on the diagram.
    assert_eq!(diagram.edges().len(), 0);
}
