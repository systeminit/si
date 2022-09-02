use dal::{test::DalContextUniversalHeadRef, StandardModel, System};
use si_test_macros::dal_test as test;

// This module is useful for local usage (executing "cargo test database") after making extensive
// changes to queries and migrations. In addition, it alphabetically comes before "integration_test"
// so it’ll fail fast in CI, but please note: tests should never be reliant on order, this is more
// of a side benefit. Even if that benefit did not exist, tests from this module could still be
// good to look for in CI failures to reduce "developer WTF time"... but that also might not be
// too useful since you'll likely see a bunch of integration tests failing anyway.

const UNSET_ID_VALUE: i64 = -1;

/// Smoke test to ensure the database is running and setup worked (migrations, etc.).
#[test]
async fn database_smoke(
    DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_, '_>,
) {
    assert!(System::get_by_id(ctx, &UNSET_ID_VALUE.into()).await.is_ok())
}
