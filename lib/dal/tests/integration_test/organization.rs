use dal::{DalContext, HistoryActor, Organization, WriteTenancy};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _history_actor = HistoryActor::SystemInit;
    let _organization = Organization::new(ctx, "iron maiden")
        .await
        .expect("cannot create organization");
}
