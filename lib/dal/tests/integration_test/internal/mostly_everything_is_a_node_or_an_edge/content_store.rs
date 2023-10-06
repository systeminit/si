use content_store::Store;
use dal::component::ComponentKind;
use dal::{DalContext, Schema};
use dal_test::test;

#[test]
async fn new(ctx: &mut DalContext) {
    // TODO(nick): replace this with something more useful. We're just trying to make sure we can
    // use both the DalContext and the store at the same time to talk to PG.
    let schema = Schema::new(ctx, "cumbersome", &ComponentKind::Standard)
        .await
        .expect("could not create schema");

    ctx.content_store()
        .lock()
        .await
        .add(schema.name())
        .expect("could not add");
    ctx.content_store()
        .lock()
        .await
        .write()
        .await
        .expect("could not write");
}
