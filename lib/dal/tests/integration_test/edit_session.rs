use crate::test_setup;
use dal::edit_session::EditSession;
use dal::test_harness::{
    create_billing_account_with_name, create_change_set, create_edit_session,
    create_visibility_change_set, create_visibility_edit_session, one_time_setup, TestContext,
};
use dal::{
    BillingAccount, ChangeSet, EditSessionStatus, HistoryActor, StandardModel, Tenancy,
    NO_EDIT_SESSION_PK,
};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _encr_key, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = ChangeSet::new(
        &txn,
        &nats,
        &tenancy,
        &history_actor,
        "create me an edit session",
        None,
    )
    .await
    .expect("cannot create changeset");

    let _edit_session = EditSession::new(
        &txn,
        &nats,
        &tenancy,
        &history_actor,
        &change_set.pk,
        "whatever",
        None,
    )
    .await
    .expect("cannot create edit session");
}

#[tokio::test]
async fn save() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _encr_key, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "blood",
    )
    .await;

    edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save edit session");

    assert_eq!(&edit_session.status, &EditSessionStatus::Saved);

    let change_set_visible = create_visibility_change_set(&change_set);

    let change_set_billing_account =
        BillingAccount::get_by_id(&txn, &tenancy, &change_set_visible, billing_account.id())
            .await
            .expect("cannot get change set billing account post edit session save")
            .expect("billing account not present in change set");

    assert_eq!(billing_account.id(), change_set_billing_account.id());
    assert_ne!(billing_account.pk(), change_set_billing_account.pk());
    assert_eq!(billing_account.name(), change_set_billing_account.name());
    assert_eq!(
        billing_account.description(),
        change_set_billing_account.description()
    );
    assert_eq!(
        change_set_billing_account.visibility().edit_session_pk,
        NO_EDIT_SESSION_PK
    );
    assert_eq!(
        billing_account.visibility().change_set_pk,
        change_set_billing_account.visibility().change_set_pk
    );
}

#[tokio::test]
async fn get_by_pk() {
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
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let result = EditSession::get_by_pk(&txn, &tenancy, &edit_session.pk)
        .await
        .expect("cannot get edit session by pk")
        .expect("edit session pk should exist");
    assert_eq!(&edit_session, &result);
}

#[tokio::test]
async fn cancel() {
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
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    edit_session
        .cancel(&txn, &nats, &history_actor)
        .await
        .expect("cannot cancel edit session");
    assert_eq!(&edit_session.status, &EditSessionStatus::Canceled);
}
