use dal::{DalContext, User, UserPk, WorkspaceSignup};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let _user = User::new(
        ctx,
        UserPk::generate(),
        "funky",
        "bobotclown@systeminit.com",
        None::<String>,
    )
    .await
    .expect("cannot create user");
}

#[test]
async fn authorize(ctx: &DalContext, nw: &WorkspaceSignup) {
    let worked = User::authorize(ctx, &nw.user.pk())
        .await
        .expect("admin group user should be authorized");
    assert!(worked, "authorized admin group user returns true");

    // TODO(theo,paulo): re-enable that when capabilities are back
    /*
    let user_no_group = User::new(
        ctx,
        UserPk::generate(),
        "funky",
        "bobotclown@systeminit.com",
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
