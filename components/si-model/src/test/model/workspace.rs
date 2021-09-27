use si_data::{NatsTxn, PgTxn};
use crate::Workspace;

use crate::test::{generate_fake_name, NewBillingAccount};

pub async fn create_workspace(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> Workspace {
    let name = generate_fake_name();
    let w = Workspace::new(
        txn,
        nats,
        name,
        &nba.billing_account.id,
        &nba.organization.id,
    )
    .await
    .expect("failed to create workspace");
    w
}
