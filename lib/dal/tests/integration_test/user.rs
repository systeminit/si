use dal::{
    BillingAccountPk, BillingAccountSignup, DalContext, JwtSecretKey, ReadTenancy, StandardModel,
    User, WorkspacePk,
};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext, bid: BillingAccountPk) {
    let _user = User::new(
        ctx,
        "funky",
        "bobotclown@systeminit.com",
        "snakesOnAPlan123",
        bid,
    )
    .await
    .expect("cannot create user");
}

#[test]
async fn login(
    ctx: &DalContext,
    bid: BillingAccountPk,
    jwt_secret_key: &JwtSecretKey,
    wid: WorkspacePk,
) {
    let password = "snakesOnAPlane123";
    let user = User::new(ctx, "funky", "bobotclown@systeminit.com", &password, bid)
        .await
        .expect("cannot create user");

    let _jwt = user
        .login(ctx, jwt_secret_key, &wid, password)
        .await
        .expect("cannot get jwt");
}

#[test]
async fn find_by_email(ctx: &mut DalContext, bid: BillingAccountPk) {
    let password = "snakesOnAPlane123";
    let user = User::new(ctx, "funky", "bobotclown@systeminit.com", &password, bid)
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

    ctx.update_read_tenancy(ReadTenancy::new(WorkspacePk::generate()));

    let fail_user = User::find_by_email(ctx, "bobotclown@systeminit.com")
        .await
        .expect("cannot find user by email");
    assert!(
        fail_user.is_none(),
        "user should not return if the tenancy is wrong"
    );
}

#[test]
async fn authorize(ctx: &DalContext, nba: &BillingAccountSignup) {
    let worked = User::authorize(ctx, nba.user.id())
        .await
        .expect("admin group user should be authorized");
    assert!(worked, "authorized admin group user returns true");

    let password = "snakesOnAPlane123";
    let user_no_group = User::new(
        ctx,
        "funky",
        "bobotclown@systeminit.com",
        &password,
        *nba.billing_account.pk(),
    )
    .await
    .expect("cannot create user");

    let f = User::authorize(ctx, user_no_group.id()).await;
    assert!(
        f.is_err(),
        "user that is not in the admin group should fail"
    );
}
