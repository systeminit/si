use crate::test::model::billing_account::NewBillingAccount;

use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use crate::{ChangeSet, EditSession, Entity, Node, Veritech};

#[allow(dead_code)]
pub async fn create_custom_entity(
    pg: &PgPool,
    txn: &PgTxn<'_>,
    nats_conn: &NatsConn,
    nats: &NatsTxn,
    veritech: &Veritech,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
    edit_session: &EditSession,
    object_type: impl AsRef<str>,
) -> Entity {
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
    let entity = Entity::for_edit_session(&txn, &node.object_id, &change_set.id, &edit_session.id)
        .await
        .expect("cannot get entity for created node");

    entity
}
