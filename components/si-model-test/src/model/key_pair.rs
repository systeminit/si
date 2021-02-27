use crate::model::billing_account::NewBillingAccount;
use names::{Generator, Name};

use si_data::{NatsTxn, PgTxn};
use si_model::KeyPair;

pub async fn create_key_pair(txn: &PgTxn<'_>, nats: &NatsTxn, nba: &NewBillingAccount) -> KeyPair {
    let key_pair = KeyPair::new(
        txn,
        nats,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        nba.billing_account.id.clone(),
    )
    .await
    .expect("cannot create key pair");
    key_pair
}
