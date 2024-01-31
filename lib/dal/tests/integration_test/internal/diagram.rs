use dal::edge::EdgeKind;
use dal::{socket::SocketEdgeKind, Connection, DalContext, Diagram, Node, Socket, StandardModel};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_node_and_check_intra_component_intelligence(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let component_bag = bagger.create_component(ctx, "13700KF", "starfield").await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "13700KF",
                "type": "component",
                "color": "#ffffff",
                "protected": false,
            },
            "domain": {
                "name": "13700KF",
            },
        }], // expected
        component_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );

    let mut node = component_bag.node(ctx).await;
    node.set_geometry(ctx, "0", "0", Some("500"), Some("500"))
        .await
        .expect("Could not set node geometry");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "13700KF",
                "type": "component",
                "color": "#ffffff",
                "protected": false,
            },
            "domain": {
                "name": "13700KF",
            },
        }], // expected
        component_bag
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value()
            .expect("could not convert to value") // actual
    );
}

#[test]
async fn get_diagram_and_create_and_delete_connection(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let fallout_bag = bagger.create_component(ctx, "tail", "fallout").await;
    let starfield_bag = bagger.create_component(ctx, "head", "starfield").await;

    let output_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationOutput,
        fallout_bag.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");
    let input_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationInput,
        starfield_bag.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

    let mut fallout_node = Node::get_by_id(ctx, &fallout_bag.node_id)
        .await
        .expect("could not find node")
        .expect("node not found");
    fallout_node
        .set_geometry(ctx, "123", "-10", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    let mut starfield_node = Node::get_by_id(ctx, &starfield_bag.node_id)
        .await
        .expect("could not find node")
        .expect("node not found");
    starfield_node
        .set_geometry(ctx, "124", "-11", Some("500"), Some("500"))
        .await
        .expect("cannot set node geometry");

    let connection = Connection::new(
        ctx,
        fallout_bag.node_id,
        *output_socket.id(),
        starfield_bag.node_id,
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
    assert_eq!(connection.id, diagram.edges()[0].edge_id());

    // Check the connection itself.
    assert_eq!(connection.source.node_id, fallout_bag.node_id);
    assert_eq!(connection.source.socket_id, *output_socket.id());
    assert_eq!(connection.destination.node_id, starfield_bag.node_id);
    assert_eq!(connection.destination.socket_id, *input_socket.id());

    // Delete the connection
    Connection::delete_for_edge(ctx, connection.id)
        .await
        .expect("could not delete connection");

    // let's reassemble the diagram to get the update
    let diagram = Diagram::assemble(ctx).await.expect("cannot find diagram");

    // Check the nodes.
    assert_eq!(diagram.components().len(), 2);

    // Check that no connections exist on the diagram.
    assert_eq!(diagram.edges().len(), 0);
}
