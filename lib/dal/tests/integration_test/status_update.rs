use dal::{DalContext, StatusUpdate};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let status_update = StatusUpdate::new(ctx)
        .await
        .expect("failed to create status update");

    assert!(status_update.dependent_values_metadata().is_empty());
    assert!(status_update.queued_dependent_value_ids().is_empty());
    assert!(status_update.running_dependent_value_ids().is_empty());
    assert!(status_update.completed_dependent_value_ids().is_empty());
}
