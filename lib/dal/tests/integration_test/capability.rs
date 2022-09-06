use dal::{BillingAccountId, Capability, DalContext, StandardModel};

use crate::dal::test;

#[test]
async fn new(ctx: &mut DalContext<'_, '_, '_>, bid: BillingAccountId) {
    ctx.update_to_billing_account_tenancies(bid);

    let _capability = Capability::new(ctx, "monkey", "*")
        .await
        .expect("cannot create capability");
}
