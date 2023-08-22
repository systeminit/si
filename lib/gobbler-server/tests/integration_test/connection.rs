use dal::workspace_snapshot::change_set::ChangeSet;
use dal::{DalContext, WorkspaceSnapshot};
use si_rabbitmq::StreamManager;
use si_test_macros::gobbler_test as test;

/// Recommended to run with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn create_snapshot(ctx: &DalContext) {
    let change_set = ChangeSet::new().expect("could not create change set");
    let _snapshot = WorkspaceSnapshot::new(ctx, &change_set)
        .await
        .expect("could not create snapshot");
}

/// Recommended to run with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
async fn connect_to_queue(_ctx: &DalContext) {
    let _ = StreamManager::new().await.expect("could not connect");
}
