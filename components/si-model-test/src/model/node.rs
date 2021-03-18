use crate::model::billing_account::NewBillingAccount;

use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use si_model::{ChangeSet, EditSession, Node, Veritech};

pub async fn create_entity_node(
    pg: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> Node {
    let node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        None,
        "leftHandPath",
        &nba.workspace.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create node");
    node
}

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
