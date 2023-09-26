//! For all tests in this file, provide "SI_TEST_BUILTIN_SCHEMAS=none" as an environment variable.

use content_store::Store;
use content_store_test::DalTestPgStore;
use dal::component::ComponentKind;
use dal::{DalContext, Schema};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let mut store = DalTestPgStore::new().await.expect("could not create store");

    // TODO(nick): replace this with something more useful. We're just trying to make sure we can
    // use both the DalContext and the store at the same time to talk to PG.
    let schema = Schema::new(ctx, "cumbersome", &ComponentKind::Standard)
        .await
        .expect("could not create schema");

    store.add(schema.name()).expect("could not add");
    store.write().await.expect("could not write");
}
