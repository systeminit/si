use crate::{test_cleanup, test_setup, DB};

use si_sdf::models::UpdateClock;

#[tokio::test]
async fn increment_for_workspace() {
    let test_account = test_setup().await.expect("failed to setup test");

    // Update clock starts at 5, because test_setup creates a system in the workspace by
    // default.
    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 5, "update count is 5");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 6, "update count is 6");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 7, "update count is 7");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
