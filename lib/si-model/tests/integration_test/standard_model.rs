use crate::test_setup;
use si_model::test_harness::{
    create_billing_account_with_name, create_change_set, create_edit_session,
    create_visibility_change_set, create_visibility_edit_session, create_visibility_head,
};
use si_model::{standard_model, BillingAccount, HistoryActor, Tenancy, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK, StandardModel};

#[tokio::test]
async fn get_by_pk() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

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

    let retrieved = standard_model::get_by_pk(&txn, "billing_accounts", billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(billing_account, retrieved);
}

#[tokio::test]
async fn get_by_id() {
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
        "coheed",
    )
    .await;

    let head_visibility = create_visibility_head();
    let change_set_visibility = create_visibility_change_set(&change_set);

    let no_head: Option<BillingAccount> = standard_model::get_by_id(
        &txn,
        "billing_accounts",
        &tenancy,
        &head_visibility,
        billing_account.id(),
    )
    .await
    .expect("could not get billing account by id");

    assert!(no_head.is_none(), "head object exists when it should not");

    let no_change_set: Option<BillingAccount> = standard_model::get_by_id(
        &txn,
        "billing_accounts",
        &tenancy,
        &change_set_visibility,
        billing_account.id(),
    )
    .await
    .expect("could not get billing account by id");
    assert!(
        no_change_set.is_none(),
        "change set object exists when it should not"
    );

    let for_edit_session: BillingAccount = standard_model::get_by_id(
        &txn,
        "billing_accounts",
        &tenancy,
        &edit_session_visibility,
        billing_account.id(),
    )
    .await
    .expect("cannot get billing account by id")
    .expect("edit session object should exist and it does not");
    assert_eq!(&for_edit_session, &billing_account);

    edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save edit session");

    let for_change_set: BillingAccount = standard_model::get_by_id(
        &txn,
        "billing_accounts",
        &tenancy,
        &change_set_visibility,
        billing_account.id(),
    )
    .await
    .expect("could not get billing account by id")
    .expect("change set object should exist but it does not");
    assert_ne!(&for_change_set.pk(), &for_edit_session.pk());
    assert_eq!(&for_change_set.id(), &for_edit_session.id());
    assert_eq!(
        &for_change_set.visibility().change_set_pk,
        &for_edit_session.visibility().change_set_pk
    );
    assert_eq!(
        &for_change_set.visibility().edit_session_pk,
        &NO_EDIT_SESSION_PK
    );

    change_set
        .apply(&txn, &nats, &history_actor)
        .await
        .expect("cannot apply change set");
    let for_head: BillingAccount = standard_model::get_by_id(
        &txn,
        "billing_accounts",
        &tenancy,
        &head_visibility,
        billing_account.id(),
    )
    .await
    .expect("could not get billing account by id")
    .expect("change set object should exist but it does not");
    assert_ne!(&for_head.pk(), &for_change_set.pk());
    assert_eq!(&for_head.id(), &for_change_set.id());
    assert_eq!(&for_head.visibility().change_set_pk, &NO_CHANGE_SET_PK,);
    assert_eq!(&for_head.visibility().edit_session_pk, &NO_EDIT_SESSION_PK);
}

#[tokio::test]
async fn list() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let mut second_edit_session =
        create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);
    let second_edit_session_visibility =
        create_visibility_edit_session(&change_set, &second_edit_session);

    let coheed_billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        "coheed",
    )
    .await;
    let spiritbox_billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        "spiritbox",
    )
    .await;
    let zeal_billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        "zeal and ardor",
    )
    .await;
    let maiden_billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &second_edit_session_visibility,
        &history_actor,
        "iron maiden",
    )
    .await;

    let head_visibility = create_visibility_head();
    let change_set_visibility = create_visibility_change_set(&change_set);

    let no_head: Vec<BillingAccount> =
        standard_model::list(&txn, "billing_accounts", &tenancy, &head_visibility)
            .await
            .expect("could not get billing account by id");
    assert_eq!(no_head.len(), 0, "there are no objects to list for head");

    let no_change_set: Vec<BillingAccount> =
        standard_model::list(&txn, "billing_accounts", &tenancy, &change_set_visibility)
            .await
            .expect("could not get billing account by id");
    assert_eq!(
        no_change_set.len(),
        0,
        "there are no objects to list for change_set"
    );

    let edit_session_set: Vec<BillingAccount> =
        standard_model::list(&txn, "billing_accounts", &tenancy, &edit_session_visibility)
            .await
            .expect("could not get billing account by id");
    assert_eq!(
        edit_session_set.len(),
        3,
        "there are 3 objects to list for edit session"
    );
    assert_eq!(
        edit_session_set,
        vec![
            coheed_billing_account.clone(),
            spiritbox_billing_account.clone(),
            zeal_billing_account.clone()
        ]
    );

    let second_edit_session_set: Vec<BillingAccount> = standard_model::list(
        &txn,
        "billing_accounts",
        &tenancy,
        &second_edit_session_visibility,
    )
    .await
    .expect("could not get billing account by id");
    assert_eq!(
        second_edit_session_set.len(),
        1,
        "there are 1 objects to list for edit session"
    );
    assert_eq!(
        second_edit_session_set,
        vec![maiden_billing_account.clone()]
    );

    edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save edit session");
    second_edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save second edit session");
    let change_set_set: Vec<BillingAccount> =
        standard_model::list(&txn, "billing_accounts", &tenancy, &change_set_visibility)
            .await
            .expect("could not get billing account by id");
    assert_eq!(
        change_set_set.len(),
        4,
        "there are 4 objects to list for edit session"
    );
    assert!(
        change_set_set
            .iter()
            .find(|ba| ba.name() == "coheed")
            .is_some(),
        "coheed is in the set"
    );
    assert!(
        change_set_set
            .iter()
            .find(|ba| ba.name() == "spiritbox")
            .is_some(),
        "spiritbox is in the set"
    );
    assert!(
        change_set_set
            .iter()
            .find(|ba| ba.name() == "zeal and ardor")
            .is_some(),
        "zeal and ardor is in the set"
    );
    assert!(
        change_set_set
            .iter()
            .find(|ba| ba.name() == "iron maiden")
            .is_some(),
        "iron maiden is in the set"
    );
}

#[tokio::test]
async fn update() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

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

    let _updated_at = standard_model::update(
        &txn,
        "billing_accounts",
        "name",
        &tenancy,
        &billing_account.pk(),
        &"funtime",
    )
    .await
    .expect("cannot update field");
}

#[tokio::test]
async fn delete() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

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

    let _updated_at =
        standard_model::delete(&txn, "billing_accounts", &tenancy, &billing_account.pk())
            .await
            .expect("cannot delete field");

    let soft_deleted: BillingAccount = standard_model::get_by_pk(&txn, "billing_accounts", &billing_account.pk())
        .await
        .expect("cannot get billing account");

    assert_eq!(soft_deleted.visibility().deleted, true);
}

#[tokio::test]
async fn delete_with_bad_tenancy() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

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

    let delete_tenancy = Tenancy::new_workspace(vec![100243]);
    let has_err =
        standard_model::delete(&txn, "billing_accounts", &delete_tenancy, billing_account.pk())
            .await;
    assert!(has_err.is_err(), "cannot delete when the tenancy is wrong");
}

#[tokio::test]
async fn undelete() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

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

    let _updated_at =
        standard_model::delete(&txn, "billing_accounts", &tenancy, &billing_account.pk())
            .await
            .expect("cannot delete field");

    let soft_deleted: BillingAccount = standard_model::get_by_pk(&txn, "billing_accounts", &billing_account.pk())
        .await
        .expect("cannot get billing account");

    assert_eq!(soft_deleted.visibility().deleted, true);

    let _updated_at =
        standard_model::undelete(&txn, "billing_accounts", &tenancy, &billing_account.pk())
            .await
            .expect("cannot delete field");

    let soft_undeleted: BillingAccount = standard_model::get_by_pk(&txn, "billing_accounts", &billing_account.pk())
        .await
        .expect("cannot get billing account");

    assert_eq!(soft_undeleted.visibility().deleted, false);
}


