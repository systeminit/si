use dal::{BillingAccountPk, DalContext, Organization};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext, bid: BillingAccountPk) {
    let _organization = Organization::new(ctx, "iron maiden", bid)
        .await
        .expect("cannot create organization");
}
