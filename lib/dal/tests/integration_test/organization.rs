use dal::DalContext;

use crate::dal::test;
use dal::test_harness::{create_change_set, create_edit_session, create_visibility_edit_session};
use dal::{HistoryActor, Organization, WriteTenancy};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let _visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _organization = Organization::new(ctx, "iron maiden")
        .await
        .expect("cannot create organization");
}
