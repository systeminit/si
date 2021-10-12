use crate::test::model::user::create_user;

use si_data::{NatsTxn, PgTxn};
use crate::{Capability, Group};

pub async fn create_group_with_users(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    name: impl Into<String>,
    user_names: Vec<String>,
    capabilities: Vec<Capability>,
    billing_account_id: impl Into<String>,
) -> Group {
    let name = name.into();
    let billing_account_id = billing_account_id.into();
    let mut user_ids: Vec<String> = Vec::new();
    for u in user_names.iter() {
        let user = create_user(
            txn,
            nats,
            u,
            format!("{}@whatevs.localdomain", u),
            &billing_account_id,
        )
        .await;
        user_ids.push(user.id);
    }
    let group = Group::new(
        &txn,
        &nats,
        name,
        user_ids,
        vec![],
        capabilities,
        billing_account_id,
    )
    .await
    .expect("cannot create group");
    group
}
