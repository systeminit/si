use crate::test_setup;
use dal::test_harness::{
    create_billing_account_with_name, create_change_set, create_edit_session,
    create_visibility_edit_session, create_visibility_head,
};
use dal::{
    BillingAccount, ChangeSet, ChangeSetStatus, HistoryActor, StandardModel, Tenancy,
    NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = ChangeSet::new(
        &txn,
        &nats,
        &tenancy,
        &history_actor,
        "mastodon rocks",
        Some(&"they are a really good band and you should like them".to_string()),
    )
    .await
    .expect("cannot create changeset");

    assert_eq!(&change_set.name, "mastodon rocks");
    assert_eq!(
        &change_set.note,
        &Some("they are a really good band and you should like them".to_string())
    );
    assert_eq!(&change_set.tenancy, &tenancy);
}

#[tokio::test]
async fn apply() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;

    let mut change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;

    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);
    let billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        "type o negative",
    )
    .await;
    edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save edit session");
    change_set
        .apply(&txn, &nats, &history_actor)
        .await
        .expect("cannot apply change set");
    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    let head_visibility = create_visibility_head();
    let head_billing_account =
        BillingAccount::get_by_id(&txn, &tenancy, &head_visibility, billing_account.id())
            .await
            .expect("cannot get billing account")
            .expect("head object should exist");
    assert_eq!(billing_account.id(), head_billing_account.id());
    assert_ne!(billing_account.pk(), head_billing_account.pk());
    assert_eq!(billing_account.name(), head_billing_account.name());
    assert_eq!(
        billing_account.description(),
        head_billing_account.description()
    );
    assert_eq!(
        head_billing_account.visibility().edit_session_pk,
        NO_EDIT_SESSION_PK
    );
    assert_eq!(
        head_billing_account.visibility().change_set_pk,
        NO_CHANGE_SET_PK,
    );
}

#[tokio::test]
async fn list_open() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;

    let a_change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let b_change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut c_change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let full_list = ChangeSet::list_open(&txn, &tenancy)
        .await
        .expect("cannot get list of open change sets");
    assert_eq!(full_list.len(), 3);
    assert!(
        full_list
            .iter()
            .find(|f| f.label == a_change_set.name)
            .is_some(),
        "change set has first entry"
    );
    assert!(
        full_list
            .iter()
            .find(|f| f.label == b_change_set.name)
            .is_some(),
        "change set has second entry"
    );
    assert!(
        full_list
            .iter()
            .find(|f| f.label == c_change_set.name)
            .is_some(),
        "change set has third entry"
    );
    c_change_set
        .apply(&txn, &nats, &history_actor)
        .await
        .expect("cannot apply change set");
    let partial_list = ChangeSet::list_open(&txn, &tenancy)
        .await
        .expect("cannot get list of open change sets");
    assert_eq!(partial_list.len(), 2);
    assert!(
        partial_list
            .iter()
            .find(|f| f.label == a_change_set.name)
            .is_some(),
        "change set has first entry"
    );
    assert!(
        partial_list
            .iter()
            .find(|f| f.label == b_change_set.name)
            .is_some(),
        "change set has second entry"
    );
}

#[tokio::test]
async fn get_by_pk() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let result = ChangeSet::get_by_pk(&txn, &tenancy, &change_set.pk)
        .await
        .expect("cannot get change set by pk")
        .expect("change set pk should exist");
    assert_eq!(&change_set, &result);
}
