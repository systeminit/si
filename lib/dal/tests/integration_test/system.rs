use crate::dal::test;
use dal::{
    test_harness::{create_system, create_workspace},
    HistoryActor, StandardModel, System, Tenancy, Visibility, WriteTenancy,
};

use crate::test_setup;

#[test]
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
        _encr_key,
    );
    let write_tenancy = WriteTenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let system = System::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "jonas-brothers-why-oh-why",
    )
    .await
    .expect("cannot create system");
    assert_eq!(system.name(), "jonas-brothers-why-oh-why");
}

#[test]
async fn set_workspace() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let system = create_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let workspace = create_workspace(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    system
        .set_workspace(&txn, &nats, &visibility, &history_actor, workspace.id())
        .await
        .expect("cannot associate system with workspace");

    let associated_workspace = system
        .workspace(&txn, &visibility)
        .await
        .expect("failed to get a workspace")
        .expect("workspace was none");
    assert_eq!(associated_workspace, workspace);
}
