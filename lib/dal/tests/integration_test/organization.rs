use dal::DalContext;

use crate::dal::test;
use dal::{HistoryActor, Organization, WriteTenancy};

#[test]
async fn new(ctx: &DalContext) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _history_actor = HistoryActor::SystemInit;
    let _organization = Organization::new(ctx, "iron maiden")
        .await
        .expect("cannot create organization");
}
