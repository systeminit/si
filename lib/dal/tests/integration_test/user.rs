use crate::test_setup;

use crate::dal::test;
use dal::test_harness::{
    billing_account_signup, create_billing_account, create_change_set, create_edit_session,
    create_visibility_edit_session, create_visibility_head,
};
use dal::{HistoryActor, ReadTenancy, StandardModel, Tenancy, User, WriteTenancy};

#[test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key
    );
    let write_tenancy = WriteTenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &(&write_tenancy).into(), &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _user = User::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        "snakesOnAPlan123",
    )
    .await
    .expect("cannot create user");
}

#[test]
async fn login() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let visibility = create_visibility_head();
    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let write_tenancy = WriteTenancy::new_billing_account(*billing_account.id());
    let password = "snakesOnAPlane123";
    let user = User::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        &password,
    )
    .await
    .expect("cannot create user");
    let _jwt = user
        .login(&txn, secret_key, billing_account.id(), password)
        .await
        .expect("cannot get jwt");
}

#[test]
async fn find_by_email() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key
    );
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let visibility = create_visibility_head();

    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let write_tenancy = WriteTenancy::new_billing_account(*billing_account.id());
    let read_tenancy = write_tenancy
        .clone_into_read_tenancy(&txn)
        .await
        .expect("unable to generate read tenancy");
    let password = "snakesOnAPlane123";
    let user = User::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        &password,
    )
    .await
    .expect("cannot create user");
    let email_user = User::find_by_email(
        &txn,
        &read_tenancy,
        &visibility,
        "bobotclown@systeminit.com",
    )
    .await
    .expect("cannot get by email");
    assert_eq!(
        Some(user),
        email_user,
        "user by email does not match created user"
    );

    let fail_user = User::find_by_email(
        &txn,
        &ReadTenancy::new_universal(),
        &visibility,
        "bobotclown@systeminit.com",
    )
    .await
    .expect("cannot find user by email");
    assert!(
        fail_user.is_none(),
        "user should not return if the tenancy is wrong"
    );
}

#[test]
async fn authorize() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);
    let history_actor = HistoryActor::SystemInit;
    let visibility = create_visibility_head();

    let (nba, _auth_token) = billing_account_signup(&txn, &nats, secret_key).await;
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.id());
    let read_tenancy = write_tenancy
        .clone_into_read_tenancy(&txn)
        .await
        .expect("unable to generate read tenancy");
    let worked = User::authorize(&txn, &read_tenancy, &visibility, nba.user.id())
        .await
        .expect("admin group user should be authorized");
    assert_eq!(worked, true, "authorized admin group user returns true");
    let password = "snakesOnAPlane123";
    let user_no_group = User::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "funky",
        "bobotclown@systeminit.com",
        &password,
    )
    .await
    .expect("cannot create user");
    let f = User::authorize(&txn, &read_tenancy, &visibility, user_no_group.id()).await;
    assert_eq!(
        f.is_err(),
        true,
        "user that is not in the admin group should fail"
    );
}
