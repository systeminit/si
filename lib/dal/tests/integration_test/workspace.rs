use crate::test_setup;

use crate::dal::test;
use dal::test_harness::{create_change_set, create_edit_session, create_visibility_edit_session};
use dal::{HistoryActor, Tenancy, Workspace};

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
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _ = Workspace::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        "iron maiden",
    )
    .await
    .expect("cannot create workspace");
}
