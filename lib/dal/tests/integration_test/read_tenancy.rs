use dal::{
    BillingAccountPk, BillingAccountSignup, DalContext, JwtSecretKey, OrganizationId, ReadTenancy,
    StandardModel, WorkspaceId, WriteTenancy,
};
use dal_test::{test, test_harness::billing_account_signup};

#[test]
async fn check_organization_specific_billing_account(
    ctx: &DalContext,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.pk()]);
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_organization_in_billing_account(
    ctx: &DalContext,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.pk());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_specific_billing_account(
    ctx: &DalContext,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.pk()]);
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_in_billing_account(
    ctx: &DalContext,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    assert_eq!(
        read_tenancy.billing_accounts(),
        vec![*nba.billing_account.pk()]
    );
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.pk());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_specific_organization(
    ctx: &DalContext,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    assert_eq!(
        read_tenancy.billing_accounts(),
        vec![*nba.billing_account.pk()]
    );
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_in_organization(
    ctx: &DalContext,
    nba: &BillingAccountSignup,
    _jwt_secret_key: &JwtSecretKey,
) {
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_identical(ctx: &DalContext) {
    let bid = BillingAccountPk::generate();
    let read_tenancy = ReadTenancy::new_billing_account(vec![bid]);
    let write_tenancy = WriteTenancy::new_billing_account(bid);

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_overlapping(ctx: &DalContext) {
    let bid = BillingAccountPk::generate();
    let read_tenancy = ReadTenancy::new_billing_account(vec![
        BillingAccountPk::generate(),
        bid,
        BillingAccountPk::generate(),
        BillingAccountPk::generate(),
        BillingAccountPk::generate(),
        BillingAccountPk::generate(),
    ]);
    let write_tenancy = WriteTenancy::new_billing_account(bid);

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_mismatched(ctx: &DalContext) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![BillingAccountPk::generate()]);
    let write_tenancy = WriteTenancy::new_billing_account(BillingAccountPk::generate());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_billing_account_pk_mismatched_level(ctx: &DalContext) {
    let read_tenancy = ReadTenancy::new_billing_account(vec![BillingAccountPk::generate()]);
    let write_tenancy = WriteTenancy::new_organization(OrganizationId::generate());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_organization_pk_identical(ctx: &DalContext, nba: &BillingAccountSignup) {
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_overlapping(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba2, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba3, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy = ReadTenancy::new_organization(
        ctx.pg_txn(),
        vec![
            *nba.organization.id(),
            *nba2.organization.id(),
            *nba3.organization.id(),
        ],
        ctx.visibility(),
    )
    .await
    .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba2.organization.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_mismatched(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy =
        ReadTenancy::new_organization(ctx.pg_txn(), vec![*nba.organization.id()], ctx.visibility())
            .await
            .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(OrganizationId::NONE);

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_pk_identical(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_overlapping(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba2, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let (nba3, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy = ReadTenancy::new_workspace(
        ctx.pg_txn(),
        vec![
            *nba.workspace.id(),
            *nba2.workspace.id(),
            *nba3.workspace.id(),
        ],
        ctx.visibility(),
    )
    .await
    .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(*nba2.workspace.id());

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_mismatched(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let (nba, _) = billing_account_signup(ctx, jwt_secret_key).await;
    let read_tenancy =
        ReadTenancy::new_workspace(ctx.pg_txn(), vec![*nba.workspace.id()], ctx.visibility())
            .await
            .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(WorkspaceId::NONE);

    let check = write_tenancy
        .check(ctx.pg_txn(), &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}
