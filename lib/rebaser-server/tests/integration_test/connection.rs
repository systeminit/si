use dal::change_set_pointer::ChangeSetPointer;
use dal::{DalContext, WorkspaceSnapshot};
use si_rabbitmq::Environment;
use si_test_macros::rebaser_test as test;

#[test]
async fn connect_to_database(ctx: &DalContext) {
    let change_set = ChangeSetPointer::new_local().expect("could not create change set");
    let _snapshot = WorkspaceSnapshot::initial(ctx, &change_set)
        .await
        .expect("could not create snapshot");
}

#[test]
async fn connect_to_queue(_ctx: &DalContext) {
    let _environment = Environment::new().await.expect("could not connect");
}
