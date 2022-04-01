use crate::dal::test;
use dal::DalContext;
use dal::{
    socket::{Socket, SocketArity, SocketEdgeKind},
    test_harness::create_socket,
    HistoryActor, Visibility, WriteTenancy,
};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let socket = Socket::new(ctx, "jane", &SocketEdgeKind::Component, &SocketArity::Many)
        .await
        .expect("cannot create schema ui menu");
    assert_eq!(socket.name(), "jane");
    assert_eq!(socket.edge_kind(), &SocketEdgeKind::Component);
    assert_eq!(socket.arity(), &SocketArity::Many);
}

#[test]
async fn set_required(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let mut socket = create_socket(ctx).await;

    socket
        .set_required(ctx, true)
        .await
        .expect("cannot set required");
    assert_eq!(socket.required(), true);
}
