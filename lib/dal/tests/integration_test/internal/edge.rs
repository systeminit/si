use dal::{
    edge::{EdgeKind, EdgeObjectId, VertexObjectKind},
    socket::SocketEdgeKind,
    Connection, DalContext, Edge, Socket, StandardModel,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
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

    let _edge = Edge::new(
        ctx,
        EdgeKind::Configuration,
        starfield_bag.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(starfield_bag.component_id),
        *input_socket.id(),
        fallout_bag.node_id,
        VertexObjectKind::Configuration,
        EdgeObjectId::from(fallout_bag.component_id),
        *output_socket.id(),
    )
    .await
    .expect("cannot create new edge");

    let parents = Edge::list_parents_for_component(ctx, starfield_bag.component_id)
        .await
        .expect("unable to find component's parents");
    assert_eq!(parents.len(), 1);
    assert_eq!(parents[0], fallout_bag.component_id);
}

#[test]
async fn create_delete_and_restore_edges(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let from_fallout = bagger.create_component(ctx, "from", "fallout").await;
    let to_starfield = bagger.create_component(ctx, "to", "starfield").await;

    let output_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationOutput,
        from_fallout.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");
    let input_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "bethesda",
        SocketEdgeKind::ConfigurationInput,
        to_starfield.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

    let connection = Connection::new(
        ctx,
        from_fallout.node_id,
        *output_socket.id(),
        to_starfield.node_id,
        *input_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    // Update the special prop
    let special_prop = from_fallout
        .find_prop(ctx, &["root", "domain", "special"])
        .await;
    from_fallout
        .update_attribute_value_for_prop(ctx, *special_prop.id(), Some(serde_json::json!["foo"]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "to",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "to",
               "attributes": "foo",
           },
        }], // expected
        to_starfield
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );

    // delete the edge
    Connection::delete_for_edge(ctx, connection.id)
        .await
        .expect("Unable to delete connection");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Check that the field of the head node is empty.
    assert_eq!(
        serde_json::json![{
            "si": {
               "name": "to",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "to",
           },
        }], // expected
        to_starfield
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );

    // restore the edge
    Connection::restore_for_edge(ctx, connection.id)
        .await
        .expect("Unable to restore connection");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Check that the value has "returned".
    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "to",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "to",
               "attributes": "foo",
           },
        }], // expected
        to_starfield
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );
}

#[test]
async fn create_multiple_connections_and_delete(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let three_bag = bagger.create_component(ctx, "three", "fallout").await;
    let new_vegas_bag = bagger.create_component(ctx, "new vegas", "fallout").await;

    let rads_prop = three_bag.find_prop(ctx, &["root", "domain", "rads"]).await;
    new_vegas_bag
        .update_attribute_value_for_prop(ctx, *rads_prop.id(), Some(serde_json::json![1]))
        .await;

    let starfield_bag = bagger
        .create_component(ctx, "destination", "starfield")
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let from_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "fallout",
        SocketEdgeKind::ConfigurationOutput,
        three_bag.node_id,
    )
    .await
    .expect("could not perform socket find")
    .expect("could not find socket");
    let to_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "fallout",
        SocketEdgeKind::ConfigurationInput,
        starfield_bag.node_id,
    )
    .await
    .expect("could not perform socket find")
    .expect("could not find socket");

    let connect_from_three = Connection::new(
        ctx,
        three_bag.node_id,
        *from_socket.id(),
        starfield_bag.node_id,
        *to_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    let connect_from_new_vegas = Connection::new(
        ctx,
        new_vegas_bag.node_id,
        *from_socket.id(),
        starfield_bag.node_id,
        *to_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    // required to happen *AFTER* the connection to trigger a dependantValuesUpdate
    three_bag
        .update_attribute_value_for_prop(ctx, *rads_prop.id(), Some(serde_json::json![2]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "destination",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "destination",
               "universe": {
                   "galaxies": [
                       {
                           "sun": "three-sun",
                           "planets": 2
                       },
                       {
                           "sun": "new vegas-sun",
                           "planets": 1
                       },
                   ],
               },
           },
        }], // expected
        starfield_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );

    // delete one of the connections
    Connection::delete_for_edge(ctx, connect_from_three.id)
        .await
        .expect("Deletion should work");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "destination",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "destination",
               "universe": {
                   "galaxies": [
                       {
                           "sun": "new vegas-sun",
                           "planets": 1
                       },
                   ],
               },
           },
        }], // expected
        starfield_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );

    // delete the other connection
    let _result = Connection::delete_for_edge(ctx, connect_from_new_vegas.id).await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "destination",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "destination",
               "universe": {
                   "galaxies": [],
               },
           },
        }], // expected
        starfield_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );
}
