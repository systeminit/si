use dal::{AttributeValueId, DalContext, StatusUpdate};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let attribute_value_id = AttributeValueId::NONE;

    let status_update = StatusUpdate::new(ctx, attribute_value_id)
        .await
        .expect("failed to create status update");

    assert_eq!(status_update.attribute_value_id(), attribute_value_id);
    assert!(status_update.dependent_values_metadata().is_empty());
    assert!(status_update.queued_dependent_value_ids().is_empty());
    assert!(status_update.running_dependent_value_ids().is_empty());
    assert!(status_update.completed_dependent_value_ids().is_empty());
}
