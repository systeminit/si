use crate::test_setup;

use dal::test_harness::{
    create_billing_account, create_change_set, create_edit_session, create_visibility_edit_session,
    create_visibility_head,
};
use dal::{HistoryActor, StandardModel, Tenancy, User};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _user = User::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        "snakesOnAPlan123",
    )
    .await
    .expect("cannot create user");
}

#[tokio::test]
async fn login() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let visibility = create_visibility_head();
    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let ba_tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);
    let password = "snakesOnAPlane123";
    let user = User::new(
        &txn,
        &nats,
        &ba_tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        &password,
    )
    .await
    .expect("cannot create user");
    let _jwt = user
        .login(&txn, &secret_key, billing_account.id(), password)
        .await
        .expect("cannot get jwt");
}

#[tokio::test]
async fn find_by_email() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let visibility = create_visibility_head();
    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let ba_tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);
    let password = "snakesOnAPlane123";
    let user = User::new(
        &txn,
        &nats,
        &ba_tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        &password,
    )
    .await
    .expect("cannot create user");
    let email_user =
        User::find_by_email(&txn, &ba_tenancy, &visibility, "bobotclown@systeminit.com")
            .await
            .expect("cannot get by email");
    assert_eq!(
        Some(user),
        email_user,
        "user by email does not match created user"
    );

    let fail_user = User::find_by_email(&txn, &tenancy, &visibility, "bobotclown@systeminit.com")
        .await
        .expect("cannot find user by email");
    assert!(
        fail_user.is_none(),
        "user should not return if the tenancy is wrong"
    );
}
