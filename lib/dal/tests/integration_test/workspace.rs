use dal::{BillingAccountPk, DalContext, Workspace};
use dal_test::test;

#[test]
async fn new(ctx: &mut DalContext, bid: BillingAccountPk) {
    let _ = Workspace::new(ctx, "iron maiden", bid)
        .await
        .expect("cannot create workspace");
}
