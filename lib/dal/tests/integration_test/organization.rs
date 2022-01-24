use crate::test_setup;

use dal::test_harness::{create_change_set, create_edit_session, create_visibility_edit_session};
use dal::{HistoryActor, Organization, Tenancy};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _organization = Organization::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "iron maiden",
    )
    .await
    .expect("cannot create organization");
}
