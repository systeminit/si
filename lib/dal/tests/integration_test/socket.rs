use dal::{
    socket::{Socket, SocketArity, SocketEdgeKind, SocketKind},
    Component, DalContext, DiagramKind, SchemaVariant, SocketId, StandardModel,
};
use dal_test::test_harness::create_schema;
use dal_test::{helpers::generate_fake_name, test};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let socket = Socket::new(
        ctx,
        "jane",
        SocketKind::Standalone,
        &SocketEdgeKind::ConfigurationOutput,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await
    .expect("cannot create schema ui menu");
    assert_eq!(socket.name(), "jane");
    assert_eq!(socket.edge_kind(), &SocketEdgeKind::ConfigurationOutput);
    assert_eq!(socket.arity(), &SocketArity::Many);
}

#[test]
async fn set_required(ctx: &DalContext) {
    let mut socket = Socket::new(
        ctx,
        generate_fake_name(),
        SocketKind::Standalone,
        &SocketEdgeKind::ConfigurationInput,
        &SocketArity::One,
        &DiagramKind::Configuration,
    )
    .await
    .expect("unable to create socket");

    socket
        .set_required(ctx, true)
        .await
        .expect("cannot set required");
    assert!(socket.required());
}

#[test]
async fn find_frame_socket_for_component(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (mut schema_variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");
    let (component, _) = Component::new(ctx, "MUSTANG GT PERFORMANCE PACK", *schema_variant.id())
        .await
        .expect("could not create component");

    // Gather what we expect. We do not exit early out of the loop in order to ensure that we get
    // the exact sockets and nothing more.
    let mut maybe_expected_output_socket_id = None;
    let mut maybe_expected_input_socket_id = None;
    for socket in schema_variant
        .sockets(ctx)
        .await
        .expect("could not get sockets")
    {
        if socket.name() == "Frame" {
            match socket.edge_kind() {
                SocketEdgeKind::ConfigurationOutput => {
                    assert!(maybe_expected_output_socket_id.is_none());
                    maybe_expected_output_socket_id = Some(*socket.id())
                }
                SocketEdgeKind::ConfigurationInput => {
                    assert!(maybe_expected_input_socket_id.is_none());
                    maybe_expected_input_socket_id = Some(*socket.id())
                }
            }
        }
    }
    let expected_output_socket_id =
        maybe_expected_output_socket_id.expect("did not find expected output socket id");
    let expected_input_socket_id =
        maybe_expected_input_socket_id.expect("did not find expected input socket id");

    // Test our query.
    let found_output_socket = Socket::find_frame_socket_for_component(
        ctx,
        *component.id(),
        SocketEdgeKind::ConfigurationOutput,
    )
    .await
    .expect("could not find frame socket for component");
    let found_input_socket = Socket::find_frame_socket_for_component(
        ctx,
        *component.id(),
        SocketEdgeKind::ConfigurationInput,
    )
    .await
    .expect("could not find frame socket for component");
    assert_eq!(
        expected_output_socket_id, // expected
        *found_output_socket.id(), // actual
    );
    assert_eq!(
        expected_input_socket_id, // expected
        *found_input_socket.id(), // actual
    );
}

#[test]
async fn list_for_component(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let (mut schema_variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");

    // Create some additional sockets from the defaults.
    Socket::new(
        ctx,
        "output",
        SocketKind::Standalone,
        &SocketEdgeKind::ConfigurationOutput,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await
    .expect("could not create socket");
    Socket::new(
        ctx,
        "input",
        SocketKind::Standalone,
        &SocketEdgeKind::ConfigurationInput,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await
    .expect("could not create socket");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");
    let (component, _) = Component::new(ctx, "MUSTANG GT PERFORMANCE PACK", *schema_variant.id())
        .await
        .expect("could not create component");

    // Gather what we expect.
    let expected_sockets = schema_variant
        .sockets(ctx)
        .await
        .expect("could not get sockets")
        .iter()
        .map(|s| *s.id())
        .collect::<Vec<SocketId>>();

    // Test our query.
    let found_sockets = Socket::list_for_component(ctx, *component.id())
        .await
        .expect("could not list for component")
        .iter()
        .map(|s| *s.id())
        .collect::<Vec<SocketId>>();
    assert_eq!(
        expected_sockets, // expected
        found_sockets,    // actual
    );
}
