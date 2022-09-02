use dal::{
    socket::{Socket, SocketArity, SocketEdgeKind, SocketKind},
    test::helpers::generate_fake_name,
    DalContext, DiagramKind,
};

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let socket = Socket::new(
        ctx,
        "jane",
        SocketKind::Provider,
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
async fn set_required(ctx: &DalContext<'_, '_, '_>) {
    let mut socket = Socket::new(
        ctx,
        generate_fake_name(),
        SocketKind::Provider,
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
