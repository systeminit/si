use dal::{DalContext, OrganizationPk, Workspace};
use dal_test::test;

#[test]
async fn new(ctx: &mut DalContext, oid: OrganizationPk) {
    let _ = Workspace::new(ctx, "iron maiden", oid)
        .await
        .expect("cannot create workspace");
}
