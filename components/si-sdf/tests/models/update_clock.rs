use crate::{test_cleanup, test_setup, DB};

use si_sdf::models::UpdateClock;

#[tokio::test]
async fn increment_for_workspace() {
    let test_account = test_setup().await.expect("failed to setup test");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 0, "update count is 0");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 1, "update count is 1");

    let update_clock = UpdateClock::create_or_update(&DB, &test_account.workspace_id, 0)
        .await
        .expect("failed to get the update clock for the workspace");
    assert_eq!(update_clock.epoch, 1, "epoch is 1");
    assert_eq!(update_clock.update_count, 2, "update count is 2");

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
