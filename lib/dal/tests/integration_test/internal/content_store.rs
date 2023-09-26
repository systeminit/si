//! For all tests in this file, provide "SI_TEST_BUILTIN_SCHEMAS=none" as an environment variable.

use content_store::PgStore;
use content_store::{ContentHash, Store};
use dal::component::ComponentKind;
use dal::{DalContext, Schema};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use si_data_pg::PgPoolConfig;

#[test]
async fn new(ctx: &DalContext) {
    let mut store = PgStore::new(ctx.pg_pool().to_owned())
        .await
        .expect("could not create store");

    let schema = Schema::new(ctx, "cumbersome", &ComponentKind::Standard)
        .await
        .expect("could not create schema");

    store.add(schema.name()).expect("could not add to store");
    store.write().await.expect("could not write");
}
