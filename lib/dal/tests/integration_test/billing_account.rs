use crate::test_setup;
use dal::test_harness::{
    billing_account_signup, create_billing_account, create_billing_account_with_name,
    create_change_set, create_edit_session, create_visibility_edit_session, create_visibility_head,
    one_time_setup, TestContext,
};
use dal::{BillingAccount, HistoryActor, StandardModel, Tenancy};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);

    let billing_account = BillingAccount::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coheed",
        Some(&"coheed and cambria".to_string()),
    )
    .await
    .expect("cannot create new billing account");

    assert_eq!(billing_account.name(), "coheed");
    assert_eq!(billing_account.description(), Some("coheed and cambria"));
    assert_eq!(billing_account.tenancy(), &tenancy);
    assert_eq!(billing_account.visibility(), &visibility);
}

#[tokio::test]
async fn get_by_pk() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats, _veritech);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coheed",
    )
    .await;

    let retrieved = BillingAccount::get_by_pk(&txn, billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(billing_account, retrieved);
}

#[tokio::test]
async fn get_by_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coheed",
    )
    .await;

    let retrieved = BillingAccount::get_by_pk(&txn, billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(billing_account, retrieved);
}

#[tokio::test]
async fn set_name() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let mut billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coheed",
    )
    .await;

    billing_account
        .set_name(&txn, &nats, &visibility, &history_actor, "woot".to_string())
        .await
        .expect("cannot set name");

    assert_eq!(billing_account.name(), "woot");
    txn.commit().await.expect("fuck");
}

#[tokio::test]
async fn set_description() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let mut billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coheed",
    )
    .await;

    billing_account
        .set_description(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            Some("smooth".to_string()),
        )
        .await
        .expect("cannot set description");
    assert_eq!(billing_account.description(), Some("smooth"));
    txn.commit().await.expect("fuck");
}

#[tokio::test]
async fn find_by_name() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let visibility = create_visibility_head();
    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let name_billing_account =
        BillingAccount::find_by_name(&txn, &tenancy, &visibility, &billing_account.name())
            .await
            .expect("cannot get by email");
    assert_eq!(
        Some(billing_account),
        name_billing_account,
        "billing_acccount by name does not match created billing account"
    );
}

#[tokio::test]
async fn get_defaults() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let (nba, _auth_token) = billing_account_signup(&txn, &nats, secret_key).await;
    let visibility = create_visibility_head();
    let tenancy = Tenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let defaults =
        BillingAccount::get_defaults(&txn, &tenancy, &visibility, nba.billing_account.id())
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
