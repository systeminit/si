use crate::test_setup;
use dal::test_harness::{
    create_billing_account, create_billing_account_with_name, create_change_set,
    create_edit_session, create_group, create_key_pair, create_schema, create_user,
    create_visibility_change_set, create_visibility_edit_session, create_visibility_head,
};
use dal::{
    standard_model, BillingAccount, Group, GroupId, HistoryActor, KeyPair, Schema, SchemaKind,
    StandardModel, Tenancy, User, UserId, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};

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
        change_set_set.iter().any(|ba| ba.name() == "coheed"),
        "coheed is in the set"
    );
    assert!(
        change_set_set.iter().any(|ba| ba.name() == "spiritbox"),
        "spiritbox is in the set"
    );
    assert!(
        change_set_set
            .iter()
            .any(|ba| ba.name() == "zeal and ardor"),
        "zeal and ardor is in the set"
    );
    assert!(
        change_set_set.iter().any(|ba| ba.name() == "iron maiden"),
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
        &visibility,
        &billing_account.id(),
        &"funtime",
        standard_model::TypeHint::Text,
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

    let soft_deleted: BillingAccount =
        standard_model::get_by_pk(&txn, "billing_accounts", &billing_account.pk())
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

    let delete_tenancy = Tenancy::new_workspace(vec![100243.into()]);
    let has_err = standard_model::delete(
        &txn,
        "billing_accounts",
        &delete_tenancy,
        billing_account.pk(),
    )
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

    let soft_deleted: BillingAccount =
        standard_model::get_by_pk(&txn, "billing_accounts", &billing_account.pk())
            .await
            .expect("cannot get billing account");

    assert_eq!(soft_deleted.visibility().deleted, true);

    let _updated_at =
        standard_model::undelete(&txn, "billing_accounts", &tenancy, &billing_account.pk())
            .await
            .expect("cannot delete field");

    let soft_undeleted: BillingAccount =
        standard_model::get_by_pk(&txn, "billing_accounts", &billing_account.pk())
            .await
            .expect("cannot get billing account");

    assert_eq!(soft_undeleted.visibility().deleted, false);
}

#[tokio::test]
async fn set_belongs_to() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let first_billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "coheed",
    )
    .await;
    let second_billing_account = create_billing_account_with_name(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "cambria",
    )
    .await;
    let key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    standard_model::set_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        key_pair.id(),
        first_billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    // You can replace the existing belongs to relationship by calling it again with a new id
    standard_model::set_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        key_pair.id(),
        second_billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");
}

#[tokio::test]
async fn unset_belongs_to() {
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

    let key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    standard_model::set_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    standard_model::unset_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        key_pair.id(),
    )
    .await
    .expect("cannot set billing account for key pair");
}

#[tokio::test]
async fn belongs_to() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let mut change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let mut edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
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
    let key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    standard_model::set_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let visibility_head = create_visibility_head();
    let no_head: Option<BillingAccount> = standard_model::belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_head,
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(no_head.is_none(), "head relationship should not exist");

    let visibility_change_set = create_visibility_change_set(&change_set);
    let no_change_set: Option<BillingAccount> = standard_model::belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_change_set,
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(
        no_change_set.is_none(),
        "change set relationship should not exist"
    );

    let edit_session_ba: BillingAccount = standard_model::belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair")
    .expect("billing account should exist for key pair");
    assert_eq!(&billing_account, &edit_session_ba);

    edit_session
        .save(&txn, &nats, &history_actor)
        .await
        .expect("cannot save edit session");
    let has_change_set: Option<BillingAccount> = standard_model::belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_change_set,
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(
        has_change_set.is_some(),
        "change set relationship should exist"
    );

    change_set
        .apply(&txn, &nats, &history_actor)
        .await
        .expect("cannot apply change set");

    let has_head: Option<BillingAccount> = standard_model::belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_head,
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(has_head.is_some(), "head relationship should exist");

    standard_model::unset_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_head,
        key_pair.id(),
    )
    .await
    .expect("cannot set billing account for key pair");
    let has_head: Option<BillingAccount> = standard_model::belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_head,
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(
        has_head.is_none(),
        "head relationship should no longer exist"
    );
}

#[tokio::test]
async fn has_many() {
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
    let a_key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    standard_model::set_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        a_key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let b_key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    standard_model::set_belongs_to(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        b_key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let visibility_head = create_visibility_head();
    let no_head: Vec<KeyPair> = standard_model::has_many(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_head,
        "key_pairs",
        billing_account.id(),
    )
    .await
    .expect("cannot get key pairs for billing account");
    assert_eq!(no_head.len(), 0, "head relationship should not exist");

    let visibility_change_set = create_visibility_change_set(&change_set);
    let no_change_set: Vec<KeyPair> = standard_model::has_many(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility_change_set,
        "key_pairs",
        billing_account.id(),
    )
    .await
    .expect("cannot get key pairs for billing account");
    assert_eq!(
        no_change_set.len(),
        0,
        "change set relationship should not exist"
    );

    txn.commit().await.expect("cannot save txn");
    let txn = conn.transaction().await.expect("cannot open txn");

    let key_pairs: Vec<KeyPair> = standard_model::has_many(
        &txn,
        "key_pair_belongs_to_billing_account",
        &tenancy,
        &visibility,
        "key_pairs",
        billing_account.id(),
    )
    .await
    .expect("cannot get key pair for billing account");
    assert_eq!(&key_pairs, &vec![a_key_pair, b_key_pair]);
}

#[tokio::test]
async fn associate_many_to_many() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let group = create_group(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let user_one = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let user_two = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
}

#[tokio::test]
async fn disassociate_many_to_many() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let group = create_group(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let user_one = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let user_two = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::disassociate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");
}

#[tokio::test]
async fn many_to_many() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let group_one = create_group(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let group_two = create_group(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let user_one = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let user_two = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group_one.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group_one.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let right_object_id: Option<&UserId> = None;
    let left_object_id: Option<&GroupId> = None;
    let group_users: Vec<User> = standard_model::many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        "groups",
        "users",
        Some(group_one.id()),
        right_object_id,
    )
    .await
    .expect("cannot get list of users for group");
    assert_eq!(group_users, vec![user_one.clone(), user_two.clone()]);

    let user_one_groups: Vec<Group> = standard_model::many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        "groups",
        "users",
        left_object_id,
        Some(user_one.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_one_groups, vec![group_one.clone()]);

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        "groups",
        "users",
        left_object_id,
        Some(user_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_two_groups, vec![group_one.clone(), group_two.clone()]);

    standard_model::disassociate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        "groups",
        "users",
        left_object_id,
        Some(user_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_two_groups, vec![group_one.clone()]);

    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        "groups",
        "users",
        left_object_id,
        Some(user_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_two_groups, vec![group_one.clone(), group_two.clone()]);
}

#[tokio::test]
async fn associate_many_to_many_no_repeat_entries() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let group = create_group(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let user_one = create_user(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    let result = standard_model::associate_many_to_many(
        &txn,
        "group_many_to_many_users",
        &tenancy,
        &visibility,
        group.id(),
        user_one.id(),
    )
    .await;
    assert!(result.is_err(), "should error");
}

#[tokio::test]
async fn find_by_attr() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats);

    let universal_tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let head_visibility = create_visibility_head();
    let billing_account = create_billing_account(
        &txn,
        &nats,
        &universal_tenancy,
        &head_visibility,
        &history_actor,
    )
    .await;
    let tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);

    let schema_one = create_schema(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        &SchemaKind::Concept,
    )
    .await;
    let schema_two = create_schema(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        &SchemaKind::Concept,
    )
    .await;
    let schema_three = create_schema(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        &SchemaKind::Concept,
    )
    .await;

    let result: Vec<Schema> = standard_model::find_by_attr(
        &txn,
        "schemas",
        &tenancy,
        &edit_session_visibility,
        "name",
        &schema_one.name().to_string(),
    )
    .await
    .expect("cannot find the object by name");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], schema_one);

    let schema_four = Schema::new(
        &txn,
        &nats,
        &tenancy,
        &edit_session_visibility,
        &history_actor,
        schema_one.name(),
        schema_one.kind(),
    )
    .await
    .expect("cannot create another schema with the same name");

    let result: Vec<Schema> = standard_model::find_by_attr(
        &txn,
        "schemas",
        &tenancy,
        &edit_session_visibility,
        "name",
        &schema_one.name().to_string(),
    )
    .await
    .expect("cannot find the object by name");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], schema_one);
    assert_eq!(result[1], schema_four);

    let result: Vec<Schema> = standard_model::find_by_attr(
        &txn,
        "schemas",
        &tenancy,
        &edit_session_visibility,
        "kind",
        &schema_one.kind().to_string(),
    )
    .await
    .expect("cannot find the object by name");
    assert_eq!(result[0], schema_one);
    assert_eq!(result[1], schema_two);
    assert_eq!(result[2], schema_three);
    assert_eq!(result[3], schema_four);
    assert_eq!(result.len(), 4);
}
