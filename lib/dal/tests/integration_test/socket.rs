use crate::dal::test;
use dal::test::helpers::generate_fake_name;
use dal::DalContext;
use dal::{
    socket::{Socket, SocketArity, SocketEdgeKind},
    SchematicKind,
};

pub mod input;

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let socket = Socket::new(
        ctx,
        "jane",
        &SocketEdgeKind::Component,
        &SocketArity::Many,
        &SchematicKind::Component,
    )
    .await
    .expect("cannot create schema ui menu");
    assert_eq!(socket.name(), "jane");
    assert_eq!(socket.edge_kind(), &SocketEdgeKind::Component);
    assert_eq!(socket.arity(), &SocketArity::Many);
}

#[test]
async fn set_required(ctx: &DalContext<'_, '_>) {
    let mut socket = Socket::new(
        ctx,
        generate_fake_name(),
        &SocketEdgeKind::Configures,
        &SocketArity::One,
        &SchematicKind::Component,
    )
    .await
    .expect("unable to create socket");

    socket
        .set_required(ctx, true)
        .await
        .expect("cannot set required");
    assert_eq!(socket.required(), true);
}
