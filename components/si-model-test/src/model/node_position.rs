use crate::NewBillingAccount;
use si_data::{NatsTxn, PgTxn};
use si_model::NodePosition;

pub async fn create_node_position(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    node_id: impl AsRef<str>,
    context_id: impl AsRef<str>,
    nba: &NewBillingAccount,
) -> NodePosition {
    let node_position = NodePosition::new(
        &txn,
        &nats,
        node_id.as_ref(),
        context_id.as_ref(),
        "0",
        "0",
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new node position");

    node_position
}
