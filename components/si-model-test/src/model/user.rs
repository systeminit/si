use si_data::{NatsTxn, PgTxn};
use si_model::User;

pub async fn create_user(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    name: impl Into<String>,
    email: impl Into<String>,
    billing_account_id: impl Into<String>,
) -> User {
    let name = name.into();
    let email = email.into();
    let billing_account_id = billing_account_id.into();

    let user = User::new(txn, nats, name, email, "superdopestar", billing_account_id)
        .await
        .expect("cannot create user");
    user
}
