use si_model_test::{
    create_change_set, create_edit_session, one_time_setup, signup_new_billing_account,
    NewBillingAccount, TestContext,
};

use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use si_model::{ChangeSet, EditSession, Node, Veritech};

#[allow(dead_code)]
pub async fn create_custom_node(
    pg: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    object_type: impl AsRef<str>,
) -> Node {
    let object_type = object_type.as_ref();
    let node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        object_type,
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create node");
    node
}

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        Some("sepultura".to_string()),
        "service",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create node");

    assert_eq!(&node.object_id.starts_with("entity:"), &true);
    assert_eq!(&node.positions, &std::collections::HashMap::new());
    assert_eq!(&node.object_type, "service");
}
