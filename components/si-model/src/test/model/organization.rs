use si_data::{NatsTxn, PgTxn};
use crate::Organization;

pub async fn create_test_organization(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    name: impl Into<String>,
    billing_account_id: impl Into<String>,
) -> Organization {
    let name = name.into();
    let billing_account_id = billing_account_id.into();
    let o = Organization::new(txn, nats, name, billing_account_id)
        .await
        .expect("failed to create organization");
    o
}
