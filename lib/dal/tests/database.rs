use dal::{test::DalContextUniversalHeadRef, StandardModel, System};
use si_test_macros::dal_test as test;

const UNSET_ID_VALUE: i64 = -1;

/// Smoke test to ensure database setup worked.
#[test]
async fn smoke(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    assert!(System::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &UNSET_ID_VALUE.into(),
    )
    .await
    .is_ok())
}
