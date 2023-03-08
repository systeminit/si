use dal::{BillingAccountSignup, DalContext, JwtSecretKey, User, WorkspacePk};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
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
async fn login(ctx: &DalContext, jwt_secret_key: &JwtSecretKey, wid: WorkspacePk) {
    let password = "snakesOnAPlane123";
    let user = User::new(ctx, "funky", "bobotclown@systeminit.com", &password)
        .await
        .expect("cannot create user");

    let _jwt = user
        .login(ctx, jwt_secret_key, &wid, password)
        .await
        .expect("cannot get jwt");
}

#[test]
async fn find_by_email(ctx: &mut DalContext) {
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
}

#[test]
async fn authorize(ctx: &DalContext, nba: &BillingAccountSignup) {
    let worked = User::authorize(ctx, &nba.user.pk())
        .await
        .expect("admin group user should be authorized");
    assert!(worked, "authorized admin group user returns true");

    // TODO(theo,paulo): re-enable that when capabilities are back
    /*
    let password = "snakesOnAPlane123";
    let user_no_group = User::new(
        ctx,
        "funky",
        "bobotclown@systeminit.com",
        &password,
    )
    .await
    .expect("cannot create user");

    let f = User::authorize(ctx, &user_no_group.pk()).await;
    assert!(
        f.is_err(),
        "user that is not in the admin group should fail"
    );
    */
}
