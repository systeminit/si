use dal::{DalContext, JwtSecretKey, ReadTenancy, WorkspacePk, WriteTenancy};
use dal_test::{test, test_harness::billing_account_signup};

#[test]
async fn check_workspace_pk_identical(ctx: &mut DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy = ReadTenancy::new(*nba.workspace.pk());
    let write_tenancy = WriteTenancy::new(*nba.workspace.pk());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_mismatched(ctx: &mut DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy = ReadTenancy::new(*nba.workspace.pk());
    let write_tenancy = WriteTenancy::new(WorkspacePk::NONE);

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}
