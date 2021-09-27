use crate::test::model::billing_account::NewBillingAccount;
use crate::test::SETTINGS;

use si_data::{NatsTxn, PgTxn};
use crate::{ApiClient, ApiClientKind};

pub async fn create_api_client(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
) -> (ApiClient, String) {
    let mut name_generator = names::Generator::with_naming(names::Name::Numbered);
    let name = name_generator.next().unwrap();

    let (api_client, token) = ApiClient::new(
        &txn,
        &nats,
        &SETTINGS.jwt_encrypt.key,
        name,
        ApiClientKind::Cli,
        &nba.billing_account.id,
    )
    .await
    .expect("cannot create new api client");
    (api_client, token)
}
