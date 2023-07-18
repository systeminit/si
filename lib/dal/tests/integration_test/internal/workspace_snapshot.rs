use dal::DalContext;
use dal_test::test;
// use dal::workspace_snapshot::WorkspaceSnapshotGraph;
// use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(_ctx: &DalContext) {
    // let snapshot = WorkspaceSnapshot::new(ctx)
    //     .await
    //     .expect("could not create snapshot");
    // let snapshot_raw: WorkspaceSnapshotGraph = snapshot
    //     .snapshot()
    //     .expect("could not get the inner snapshot");
    //
    // assert!(snapshot_raw.is_directed());
    // assert_eq!(
    //     1,                         // expected
    //     snapshot_raw.node_count()  // actual
    // );
    // assert_eq!(
    //     0,                         // expected
    //     snapshot_raw.edge_count()  // actual
    // );
    dbg!("foo");
}
