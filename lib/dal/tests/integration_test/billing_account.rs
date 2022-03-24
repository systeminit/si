use dal::{
    test::{
        helpers::{create_billing_account, generate_fake_name},
        DalContextUniversalHeadRef,
    },
    BillingAccount, BillingAccountSignup, DalContext, StandardModel,
};

use crate::dal::test;

#[test]
async fn new(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    let name = generate_fake_name();
    let billing_account = BillingAccount::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        &name,
        Some(&"coheed and cambria".to_string()),
    )
    .await
    .expect("cannot create new billing account");

    assert_eq!(billing_account.name(), &name);
    assert_eq!(billing_account.description(), Some("coheed and cambria"));
    assert_eq!(billing_account.tenancy(), &ctx.write_tenancy().into());
    assert_eq!(billing_account.visibility(), ctx.visibility());
}

#[test]
async fn get_by_pk(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    let billing_account = create_billing_account(ctx).await;

    let retrieved = BillingAccount::get_by_pk(ctx.pg_txn(), billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(billing_account, retrieved);
}

#[test]
async fn get_by_id(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    let billing_account = create_billing_account(ctx).await;

    let retrieved = BillingAccount::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        billing_account.id(),
    )
    .await
    .expect("cannot get billing account by id")
    .expect("there was no billing account by id");

    assert_eq!(billing_account, retrieved);
}

#[test]
async fn set_name(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    let mut billing_account = create_billing_account(ctx).await;

    let new_name = generate_fake_name();
    billing_account
        .set_name(
            ctx.pg_txn(),
            ctx.nats_txn(),
            ctx.visibility(),
            ctx.history_actor(),
            new_name.clone(),
        )
        .await
        .expect("cannot set name");

    assert_eq!(billing_account.name(), &new_name);
}

#[test]
async fn set_description(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    let mut billing_account = create_billing_account(ctx).await;

    billing_account
        .set_description(
            ctx.pg_txn(),
            ctx.nats_txn(),
            ctx.visibility(),
            ctx.history_actor(),
            Some("smooth".to_string()),
        )
        .await
        .expect("cannot set description");
    assert_eq!(billing_account.description(), Some("smooth"));
}

#[test]
async fn find_by_name(DalContextUniversalHeadRef(ctx): DalContextUniversalHeadRef<'_, '_, '_>) {
    let billing_account = create_billing_account(ctx).await;

    let name_billing_account = BillingAccount::find_by_name(
        ctx.pg_txn(),
        ctx.read_tenancy(),
        ctx.visibility(),
        &billing_account.name(),
    )
    .await
    .expect("cannot get by email");
    assert_eq!(
        Some(billing_account),
        name_billing_account,
        "billing_acccount by name does not match created billing account"
    );
}

#[test]
async fn get_defaults(ctx: &mut DalContext<'_, '_>, nba: &BillingAccountSignup) {
    ctx.update_to_billing_account_tenancies(*nba.billing_account.id());

    let defaults = BillingAccount::get_defaults(
        ctx.pg_txn(),
        ctx.read_tenancy(),
        ctx.visibility(),
        nba.billing_account.id(),
    )
    .await
    .expect("cannot get defaults for billing account");
    assert_eq!(
        defaults.organization, nba.organization,
        "default organization matches created organization"
    );
    assert_eq!(
        defaults.workspace, nba.workspace,
        "default workspace matches created workspace"
    );
}
