use dal::{BillingAccountId, Capability, DalContext, StandardModel};
use dal_test::test;

#[test]
async fn new(ctx: &mut DalContext, bid: BillingAccountId) {
    ctx.update_to_billing_account_tenancies(bid);

    let _capability = Capability::new(ctx, "monkey", "*")
        .await
        .expect("cannot create capability");
}
