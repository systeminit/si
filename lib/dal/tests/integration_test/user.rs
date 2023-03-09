use dal::{DalContext, JwtSecretKey, Tenancy, User, WorkspaceSignup};
use dal_test::{test, test_harness::create_workspace};

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
async fn login(ctx: &DalContext, jwt_secret_key: &JwtSecretKey) {
    let password = "snakesOnAPlane123";
    let user = User::new(ctx, "funky", "bobotclown@systeminit.com", &password)
        .await
        .expect("cannot create user");

    let _jwt = user
        .login(ctx, jwt_secret_key, password)
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

    let workspace = create_workspace(ctx).await;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));

    let email_user = User::find_by_email(ctx, "bobotclown@systeminit.com")
        .await
        .expect("cannot get by email");
    assert!(
        email_user.is_none(),
        "wrong tenancy, user should not be found",
    );

    ctx.update_tenancy(Tenancy::new_empty());

    let email_user = User::find_by_email(ctx, "bobotclown@systeminit.com")
        .await
        .expect("cannot get by email");
    assert!(email_user.is_none(), "no tenancy, user should not be found",);
}

#[test]
async fn authorize(ctx: &DalContext, nw: &WorkspaceSignup) {
    let worked = User::authorize(ctx, &nw.user.pk())
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
