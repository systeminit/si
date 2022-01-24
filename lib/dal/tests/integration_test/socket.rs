use dal::{
    socket::{Socket, SocketArity, SocketEdgeKind},
    test_harness::create_socket,
    HistoryActor, Tenancy, Visibility,
};

use crate::test_setup;

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let socket = Socket::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "jane",
        &SocketEdgeKind::Component,
        &SocketArity::Many,
    )
    .await
    .expect("cannot create schema ui menu");
    assert_eq!(socket.name(), "jane");
    assert_eq!(socket.edge_kind(), &SocketEdgeKind::Component);
    assert_eq!(socket.arity(), &SocketArity::Many);
}

#[tokio::test]
async fn set_required() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let mut socket = create_socket(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    socket
        .set_required(&txn, &nats, &visibility, &history_actor, true)
        .await
        .expect("cannot set required");
    assert_eq!(socket.required(), true);
}
