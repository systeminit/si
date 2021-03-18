use crate::model::billing_account::NewBillingAccount;

use si_data::{NatsTxn, PgTxn};
use si_model::{ChangeSet, EditSession};

pub async fn create_edit_session(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    change_set: &ChangeSet,
) -> EditSession {
    EditSession::new(
        &txn,
        &nats,
        None,
        change_set.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("Cannot create new edit session")
}
