use dal::DalContext;
use dal_test::test;

#[test]
async fn cycle_check_guard_test(ctx: &DalContext) {
    let snap = ctx.workspace_snapshot().expect("get snap");
    let guard = snap.enable_cycle_check().await;

    assert!(snap.cycle_check().await);

    drop(guard);

    assert!(!snap.cycle_check().await);
}
