use dal::change_status::ChangeStatus;
use dal::edge::EdgeKind;
use dal::{
    socket::SocketEdgeKind, Component, ComponentView, ComponentViewProperties, Connection,
    DalContext, Diagram, DiagramEdgeView, Node, Schema, Socket, StandardModel,
};
use dal_test::helpers::component_payload::ComponentPayloadAssembler;
use dal_test::test;
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
async fn get_diagram_and_create_and_delete_connection(ctx: &DalContext) {
    let mut assembler = ComponentPayloadAssembler::new();
    let fallout_payload = assembler.create_component(ctx, "tail", "fallout").await;
    let starfield_payload = assembler.create_component(ctx, "head", "starfield").await;

    let output_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationOutput,
        fallout_payload.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");
    let input_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationInput,
        starfield_payload.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

    let mut fallout_node = Node::get_by_id(ctx, &fallout_payload.node_id)
        .await
        .expect("could not find node")
        .expect("node not found");
    fallout_node
        .set_geometry(ctx, "123", "-10", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    let mut starfield_node = Node::get_by_id(ctx, &starfield_payload.node_id)
        .await
        .expect("could not find node")
        .expect("node not found");
    starfield_node
        .set_geometry(ctx, "124", "-11", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    let connection = Connection::new(
        ctx,
        fallout_payload.node_id,
        *output_socket.id(),
        starfield_payload.node_id,
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

    assert_eq!(source_node_id, fallout_payload.node_id);
    assert_eq!(source_socket_id, *output_socket.id());
    assert_eq!(destination_node_id, starfield_payload.node_id);
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
