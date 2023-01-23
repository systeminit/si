use dal::{BillingAccount, BillingAccountSignup, DalContext};
use dal_test::{
    helpers::{create_billing_account, generate_fake_name},
    test,
};

#[test]
async fn new(ctx: &DalContext) {
    let name = generate_fake_name();
    let billing_account = BillingAccount::new(ctx, &name, Some(&"coheed and cambria".to_string()))
        .await
        .expect("cannot create new billing account");

    assert_eq!(billing_account.name(), &name);
    assert_eq!(
        billing_account.description().as_deref(),
        Some("coheed and cambria")
    );
}

#[test]
async fn get_by_pk(ctx: &DalContext) {
    let billing_account = create_billing_account(ctx).await;

    let retrieved = BillingAccount::get_by_pk(ctx, billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(billing_account, retrieved);
}

#[test]
async fn find_by_name(ctx: &DalContext) {
    let billing_account = create_billing_account(ctx).await;

    let name_billing_account = BillingAccount::find_by_name(ctx, &billing_account.name())
        .await
        .expect("cannot get by email");
    assert_eq!(
        Some(billing_account),
        name_billing_account,
        "billing_acccount by name does not match created billing account"
    );
}

#[test]
async fn get_defaults(ctx: &mut DalContext, nba: &BillingAccountSignup) {
    let defaults = BillingAccount::get_defaults(ctx, nba.billing_account.pk())
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
