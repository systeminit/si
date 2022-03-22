use crate::test_setup;

use crate::dal::test;
use dal::test_harness::{create_change_set, create_edit_session, create_visibility_edit_session};
use dal::{Capability, HistoryActor, WriteTenancy};

#[test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key
    );
    let write_tenancy = WriteTenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &(&write_tenancy).into(), &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _capability = Capability::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "monkey",
        "*",
    )
    .await
    .expect("cannot create capability");
}
