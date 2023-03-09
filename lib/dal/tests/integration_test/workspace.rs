use dal::{DalContext, Workspace};
use dal_test::test;

#[test]
async fn new(ctx: &mut DalContext) {
    let _ = Workspace::new(ctx, "iron maiden")
        .await
        .expect("cannot create workspace");
}
