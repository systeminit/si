use dal::{
    BillingAccountId, BillingAccountSignup, DalContext, JwtSecretKey, ReadTenancy, StandardModel,
    User,
};

use crate::dal::test;

#[test]
async fn new(ctx: &mut DalContext<'_, '_>, bid: BillingAccountId) {
    ctx.update_to_billing_account_tenancies(bid);

    let _user = User::new(
        ctx,
        "funky",
        "bobotclown@systeminit.com",
        "snakesOnAPlan123",
    )
    .await
    .expect("cannot create user");
}

#[test]
async fn login(ctx: &mut DalContext<'_, '_>, bid: BillingAccountId, jwt_secret_key: &JwtSecretKey) {
    ctx.update_to_billing_account_tenancies(bid);

    let password = "snakesOnAPlane123";
    let user = User::new(ctx, "funky", "bobotclown@systeminit.com", &password)
        .await
        .expect("cannot create user");

    let _jwt = user
        .login(ctx, jwt_secret_key, &bid, password)
        .await
        .expect("cannot get jwt");
}

#[test]
async fn find_by_email(ctx: &mut DalContext<'_, '_>, bid: BillingAccountId) {
    ctx.update_to_billing_account_tenancies(bid);

    let password = "snakesOnAPlane123";
    let user = User::new(ctx, "funky", "bobotclown@systeminit.com", &password)
        .await
        .expect("cannot create user");

    let email_user = User::find_by_email(ctx, "bobotclown@systeminit.com")
        .await
        .expect("cannot get by email");
    assert_eq!(
        Some(user),
        email_user,
        "user by email does not match created user"
    );

    ctx.update_read_tenancy(ReadTenancy::new_universal());

    let fail_user = User::find_by_email(ctx, "bobotclown@systeminit.com")
        .await
        .expect("cannot find user by email");
    assert!(
        fail_user.is_none(),
        "user should not return if the tenancy is wrong"
    );
}

#[test]
async fn authorize(ctx: &mut DalContext<'_, '_>, nba: &BillingAccountSignup) {
    ctx.update_to_billing_account_tenancies(*nba.billing_account.id());

    let worked = User::authorize(ctx, nba.user.id())
        .await
        .expect("admin group user should be authorized");
    assert_eq!(worked, true, "authorized admin group user returns true");

    let password = "snakesOnAPlane123";
    let user_no_group = User::new(ctx, "funky", "bobotclown@systeminit.com", &password)
        .await
        .expect("cannot create user");

    let f = User::authorize(ctx, user_no_group.id()).await;
    assert_eq!(
        f.is_err(),
        true,
        "user that is not in the admin group should fail"
    );
}
