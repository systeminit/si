use dal::{BillingAccountId, Capability, DalContext, StandardModel};

use crate::dal::test;

#[test]
async fn new(ctx: &mut DalContext<'_, '_>, bid: BillingAccountId) {
    ctx.update_to_billing_account_tenancies(bid);

    let _capability = Capability::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        "monkey",
        "*",
    )
    .await
    .expect("cannot create capability");
}
