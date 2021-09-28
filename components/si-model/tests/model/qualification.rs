use si_data::{NatsTxn, PgTxn};
use si_model::{ChangeSet, EditSession, Node, Qualification};
use si_model::test::{
    create_change_set, create_custom_node, create_edit_session, one_time_setup,
    signup_new_billing_account, NewBillingAccount, TestContext,
};

async fn create_qualification(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    entity_id: impl AsRef<str>,
    name: impl AsRef<str>,
    qualified: bool,
) -> Qualification {
    let entity_id = entity_id.as_ref();
    let name = name.as_ref();
    let q = Qualification::new(
        &txn,
        &nats,
        entity_id,
        name,
        qualified,
        Some("was great".to_string()),
        None,
        &change_set.id,
        &edit_session.id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new qualification");
    q
}

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let node: Node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "dockerImage",
    )
    .await;

    let q = Qualification::new(
        &txn,
        &nats,
        &node.object_id,
        "soundgarden",
        true,
        Some("was great".to_string()),
        None,
        &change_set.id,
        &edit_session.id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new qualification");

    assert_eq!(q.entity_id, node.object_id);
}

#[tokio::test]
async fn for_edit_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let node: Node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        "dockerImage",
    )
    .await;

    let qa = create_qualification(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &node.object_id,
        "soundgarden",
        true,
    )
    .await;

    let qb = create_qualification(
        &txn,
        &nats,
        &nba,
        &change_set,
        &edit_session,
        &node.object_id,
        "prince",
        true,
    )
    .await;

    let qualifications =
        Qualification::for_edit_session(&txn, &node.object_id, &change_set.id, &edit_session.id)
            .await
            .expect("cannot get qualifications for edit session");

    assert_eq!(qualifications, vec![qa, qb]);
}
