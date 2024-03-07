use dal::{DalContext, Schema};
use dal_test::test;

mod variant;

#[test]
async fn new(ctx: &DalContext) {
    let _schema = Schema::new(ctx, "mastodon")
        .await
        .expect("cannot create schema");
}
