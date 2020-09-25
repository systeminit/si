use crate::{test_cleanup, test_setup, DB};

use si_sdf::models::UpdateClock;

#[tokio::test]
async fn increment_for_workspace() {
    let test_account = test_setup().await.expect("failed to setup test");

    // Update clock starts at 4, because test_setup creates a system in the workspace by
    // default.
    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 4, "update count is 4");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 5, "update count is 1");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 6, "update count is 2");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
