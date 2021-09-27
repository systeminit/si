use crate::model::billing_account::NewBillingAccount;

use si_data::{NatsTxn, PgTxn};
use si_model::ChangeSet;

pub async fn create_change_set(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> ChangeSet {
    ChangeSet::new(&txn, &nats, None, nba.workspace.id.clone())
        .await
        .expect("cannot create change_set")
}
