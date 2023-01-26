use dal::{Capability, DalContext};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let _capability = Capability::new(ctx, "monkey", "*")
        .await
        .expect("cannot create capability");
}
