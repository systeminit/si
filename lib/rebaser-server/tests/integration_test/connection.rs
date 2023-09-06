use dal::workspace_snapshot::change_set::ChangeSet;
use dal::{DalContext, WorkspaceSnapshot};
use si_rabbitmq::Environment;
use si_test_macros::rebaser_test as test;

#[test]
async fn connect_to_database(ctx: &DalContext) {
    let change_set = ChangeSet::new().expect("could not create change set");
    let _snapshot = WorkspaceSnapshot::new(ctx, &change_set)
        .await
        .expect("could not create snapshot");
}

#[test]
async fn connect_to_queue(_ctx: &DalContext) {
    let _environment = Environment::new().await.expect("could not connect");
}
