use dal::DalContext;

use crate::dal::test;

use dal::Workspace;

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let _ = Workspace::new(ctx, "iron maiden")
        .await
        .expect("cannot create workspace");
}
