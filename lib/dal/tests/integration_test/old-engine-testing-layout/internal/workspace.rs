use dal::{DalContext, Workspace, WorkspacePk};
use dal_test::test;

#[test]
async fn new(ctx: &mut DalContext) {
    let _ = Workspace::new(ctx, WorkspacePk::generate(), "iron maiden")
        .await
        .expect("cannot create workspace");
}
