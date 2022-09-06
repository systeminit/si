use crate::dal::test;
use dal::DalContext;
use dal::{
    test_harness::{create_system, create_workspace},
    StandardModel, System,
};

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let system = System::new(ctx, "jonas-brothers-why-oh-why")
        .await
        .expect("cannot create system");
    assert_eq!(system.name(), "jonas-brothers-why-oh-why");
}

#[test]
async fn set_workspace(ctx: &DalContext<'_, '_, '_>) {
    let system = create_system(ctx).await;
    let workspace = create_workspace(ctx).await;

    system
        .set_workspace(ctx, workspace.id())
        .await
        .expect("cannot associate system with workspace");

    let associated_workspace = system
        .workspace(ctx)
        .await
        .expect("failed to get a workspace")
        .expect("workspace was none");
    assert_eq!(associated_workspace, workspace);
}
