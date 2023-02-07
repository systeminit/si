use dal::{BillingAccountPk, DalContext, Group, StandardModel};
use dal_test::{
    helpers::{create_group, create_user},
    test,
};

#[test]
async fn new(ctx: &mut DalContext, bid: BillingAccountPk) {
    let _group = Group::new(ctx, "funky", bid)
        .await
        .expect("cannot create group");
}

#[test]
async fn add_user(ctx: &mut DalContext, bid: BillingAccountPk) {
    let group = create_group(ctx, bid).await;
    let user_one = create_user(ctx, bid).await;
    let user_two = create_user(ctx, bid).await;

    group
        .add_user(ctx, user_one.id())
        .await
        .expect("cannot add user");
    group
        .add_user(ctx, user_two.id())
        .await
        .expect("cannot add user");
}

#[test]
async fn remove_user(ctx: &mut DalContext, bid: BillingAccountPk) {
    let group = create_group(ctx, bid).await;
    let user_one = create_user(ctx, bid).await;
    let user_two = create_user(ctx, bid).await;

    group
        .add_user(ctx, user_one.id())
        .await
        .expect("cannot add user");
    group
        .add_user(ctx, user_two.id())
        .await
        .expect("cannot add user");

    group
        .remove_user(ctx, user_one.id())
        .await
        .expect("cannot remove user");
    group
        .remove_user(ctx, user_two.id())
        .await
        .expect("cannot remove user");
}

#[test]
async fn users(ctx: &mut DalContext, bid: BillingAccountPk) {
    let group = create_group(ctx, bid).await;
    let user_one = create_user(ctx, bid).await;
    let user_two = create_user(ctx, bid).await;

    group
        .add_user(ctx, user_one.id())
        .await
        .expect("cannot add user");
    group
        .add_user(ctx, user_two.id())
        .await
        .expect("cannot add user");

    let all_users = group.users(ctx).await.expect("cannot list users for group");
    assert_eq!(
        all_users
            .into_iter()
            .filter(|u| u == &user_one || u == &user_two)
            .count(),
        2,
        "all associated users in the list"
    );

    group
        .remove_user(ctx, user_one.id())
        .await
        .expect("cannot remove user");

    let some_users = group.users(ctx).await.expect("cannot list users for group");
    assert_eq!(
        some_users,
        vec![user_two.clone()],
        "only one associated user in the list"
    );
}
