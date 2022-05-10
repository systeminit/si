use crate::dal::test;
use dal::{BillingAccountSignup, DalContext};

use dal::test_harness::{
    create_billing_account, create_billing_account_with_name, create_change_set,
    create_edit_session, create_group, create_key_pair, create_schema, create_user,
    create_visibility_change_set, create_visibility_edit_session, create_visibility_head,
};
use dal::{
    component::ComponentKind, standard_model, BillingAccount, Group, GroupId, KeyPair, Schema,
    SchemaKind, StandardModel, User, UserId, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};

#[test]
async fn get_by_pk(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let retrieved = standard_model::get_by_pk(&ctx, "billing_accounts", nba.billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(nba.billing_account, retrieved);
}

#[test]
async fn get_by_id(ctx: &DalContext<'_, '_>) {
    let mut change_set = create_change_set(ctx).await;
    let mut edit_session = create_edit_session(ctx, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(edit_session_visibility);

    let billing_account = create_billing_account_with_name(&ctx, "coheed").await;

    let head_visibility = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(head_visibility);
    let change_set_visibility = create_visibility_change_set(&change_set);
    let change_set_ctx = ctx.clone_with_new_visibility(change_set_visibility);

    let no_head: Option<BillingAccount> =
        standard_model::get_by_id(&head_ctx, "billing_accounts", billing_account.id())
            .await
            .expect("could not get billing account by id");

    assert!(no_head.is_none(), "head object exists when it should not");

    let no_change_set: Option<BillingAccount> =
        standard_model::get_by_id(&change_set_ctx, "billing_accounts", billing_account.id())
            .await
            .expect("could not get billing account by id");
    assert!(
        no_change_set.is_none(),
        "change set object exists when it should not"
    );

    let for_edit_session: BillingAccount =
        standard_model::get_by_id(&ctx, "billing_accounts", billing_account.id())
            .await
            .expect("cannot get billing account by id")
            .expect("edit session object should exist and it does not");
    assert_eq!(&for_edit_session, &billing_account);

    edit_session
        .save(&ctx)
        .await
        .expect("cannot save edit session");

    let for_change_set: BillingAccount =
        standard_model::get_by_id(&change_set_ctx, "billing_accounts", billing_account.id())
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
        .apply(&change_set_ctx)
        .await
        .expect("cannot apply change set");
    let for_head: BillingAccount =
        standard_model::get_by_id(&head_ctx, "billing_accounts", billing_account.id())
            .await
            .expect("could not get billing account by id")
            .expect("change set object should exist but it does not");
    assert_ne!(&for_head.pk(), &for_change_set.pk());
    assert_eq!(&for_head.id(), &for_change_set.id());
    assert_eq!(&for_head.visibility().change_set_pk, &NO_CHANGE_SET_PK,);
    assert_eq!(&for_head.visibility().edit_session_pk, &NO_EDIT_SESSION_PK);
}

#[test]
async fn list(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let change_set = create_change_set(ctx).await;
    let mut edit_session = create_edit_session(ctx, &change_set).await;
    let mut second_edit_session = create_edit_session(ctx, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(edit_session_visibility);
    let second_edit_session_visibility =
        create_visibility_edit_session(&change_set, &second_edit_session);
    let second_edit_session_ctx = ctx.clone_with_new_visibility(second_edit_session_visibility);

    let coheed_billing_account = create_billing_account_with_name(&ctx, "coheed").await;
    let spiritbox_billing_account = create_billing_account_with_name(&ctx, "spiritbox").await;
    let zeal_billing_account = create_billing_account_with_name(&ctx, "zeal and ardor").await;
    let maiden_billing_account =
        create_billing_account_with_name(&second_edit_session_ctx, "iron maiden").await;

    let head_visibility = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(head_visibility);
    let change_set_visibility = create_visibility_change_set(&change_set);
    let change_set_ctx = ctx.clone_with_new_visibility(change_set_visibility);

    let no_head: Vec<BillingAccount> = standard_model::list(&head_ctx, "billing_accounts")
        .await
        .expect("could not get billing account by id");
    assert_eq!(
        no_head.len(),
        1,
        "there are no objects beyond the default to list for head"
    );

    let no_change_set: Vec<BillingAccount> =
        standard_model::list(&change_set_ctx, "billing_accounts")
            .await
            .expect("could not get billing account by id");
    assert_eq!(
        no_change_set.len(),
        1,
        "there are no objects beyond the default to list for change_set"
    );

    let edit_session_set: Vec<BillingAccount> = standard_model::list(&ctx, "billing_accounts")
        .await
        .expect("could not get billing account by id");
    assert_eq!(
        edit_session_set.len(),
        4,
        "there are 4 objects to list for edit session"
    );
    assert_eq!(
        edit_session_set,
        vec![
            nba.billing_account.clone(),
            coheed_billing_account.clone(),
            spiritbox_billing_account.clone(),
            zeal_billing_account.clone()
        ]
    );

    let second_edit_session_set: Vec<BillingAccount> =
        standard_model::list(&second_edit_session_ctx, "billing_accounts")
            .await
            .expect("could not get billing account by id");
    assert_eq!(
        second_edit_session_set.len(),
        2,
        "there are 2 objects to list for edit session"
    );
    assert_eq!(
        second_edit_session_set,
        vec![nba.billing_account.clone(), maiden_billing_account.clone()]
    );

    edit_session
        .save(&ctx)
        .await
        .expect("cannot save edit session");
    second_edit_session
        .save(&second_edit_session_ctx)
        .await
        .expect("cannot save second edit session");
    let change_set_set: Vec<BillingAccount> =
        standard_model::list(&change_set_ctx, "billing_accounts")
            .await
            .expect("could not get billing account by id");
    assert_eq!(
        change_set_set.len(),
        5,
        "there are 5 objects to list for edit session"
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

#[test]
async fn update(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let _updated_at = standard_model::update(
        ctx,
        "billing_accounts",
        "name",
        nba.billing_account.id(),
        &"funtime",
        standard_model::TypeHint::Text,
    )
    .await
    .expect("cannot update field");
}

#[test]
async fn delete(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let _updated_at = standard_model::delete(ctx, "billing_accounts", nba.billing_account.pk())
        .await
        .expect("cannot delete field");

    let soft_deleted: BillingAccount =
        standard_model::get_by_pk(ctx, "billing_accounts", nba.billing_account.pk())
            .await
            .expect("cannot get billing account");

    assert!(
        soft_deleted.visibility().deleted_at.is_some(),
        "should be deleted"
    );
}

#[test]
async fn undelete(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let _updated_at = standard_model::delete(ctx, "billing_accounts", nba.billing_account.pk())
        .await
        .expect("cannot delete field");

    let soft_deleted: BillingAccount =
        standard_model::get_by_pk(ctx, "billing_accounts", nba.billing_account.pk())
            .await
            .expect("cannot get billing account");

    assert!(
        soft_deleted.visibility().deleted_at.is_some(),
        "should be deleted"
    );

    let _updated_at = standard_model::undelete(ctx, "billing_accounts", nba.billing_account.pk())
        .await
        .expect("cannot delete field");

    let soft_undeleted: BillingAccount =
        standard_model::get_by_pk(ctx, "billing_accounts", nba.billing_account.pk())
            .await
            .expect("cannot get billing account");

    assert!(
        soft_undeleted.visibility().deleted_at.is_none(),
        "should be no longer deleted"
    );
}

#[test]
async fn set_belongs_to(ctx: &DalContext<'_, '_>) {
    let first_billing_account = create_billing_account_with_name(ctx, "coheed").await;
    let second_billing_account = create_billing_account_with_name(ctx, "cambria").await;
    let key_pair = create_key_pair(ctx).await;

    standard_model::set_belongs_to(
        &ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
        first_billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    // You cannot replace the existing belongs to relationship by calling it again with a new id
    match standard_model::set_belongs_to(
        &ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
        second_billing_account.id(),
    )
    .await
    {
        Err(err) => {
            assert!(err
                .to_string()
                .contains("duplicate key value violates unique constraint "));
        }
        Ok(_) => panic!("set belongs to twice should fail"),
    };
}

#[test]
async fn unset_belongs_to(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let key_pair = create_key_pair(ctx).await;

    standard_model::set_belongs_to(
        &ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
        nba.billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    standard_model::unset_belongs_to(&ctx, "key_pair_belongs_to_billing_account", key_pair.id())
        .await
        .expect("cannot set billing account for key pair");
}

#[test]
async fn belongs_to(ctx: &DalContext<'_, '_>) {
    let mut change_set = create_change_set(ctx).await;
    let mut edit_session = create_edit_session(ctx, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(visibility);
    let billing_account = create_billing_account_with_name(&ctx, "coheed").await;
    let key_pair = create_key_pair(&ctx).await;

    standard_model::set_belongs_to(
        &ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let visibility_head = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(visibility_head);
    let no_head: Option<BillingAccount> = standard_model::belongs_to(
        &head_ctx,
        "key_pair_belongs_to_billing_account",
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(no_head.is_none(), "head relationship should not exist");

    let visibility_change_set = create_visibility_change_set(&change_set);
    let change_set_ctx = ctx.clone_with_new_visibility(visibility_change_set);
    let no_change_set: Option<BillingAccount> = standard_model::belongs_to(
        &change_set_ctx,
        "key_pair_belongs_to_billing_account",
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
        &ctx,
        "key_pair_belongs_to_billing_account",
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair")
    .expect("billing account should exist for key pair");
    assert_eq!(&billing_account, &edit_session_ba);

    edit_session
        .save(&ctx)
        .await
        .expect("cannot save edit session");
    let has_change_set: Option<BillingAccount> = standard_model::belongs_to(
        &change_set_ctx,
        "key_pair_belongs_to_billing_account",
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
        .apply(&change_set_ctx)
        .await
        .expect("cannot apply change set");

    let has_head: Option<BillingAccount> = standard_model::belongs_to(
        &head_ctx,
        "key_pair_belongs_to_billing_account",
        "billing_accounts",
        key_pair.id(),
    )
    .await
    .expect("cannot get billing account for key pair");
    assert!(has_head.is_some(), "head relationship should exist");

    standard_model::unset_belongs_to(
        &head_ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
    )
    .await
    .expect("cannot set billing account for key pair");
    let has_head: Option<BillingAccount> = standard_model::belongs_to(
        &head_ctx,
        "key_pair_belongs_to_billing_account",
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

#[test]
async fn has_many(ctx: &DalContext<'_, '_>) {
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(visibility);
    let billing_account = create_billing_account_with_name(&ctx, "coheed").await;
    let a_key_pair = create_key_pair(&ctx).await;
    standard_model::set_belongs_to(
        &ctx,
        "key_pair_belongs_to_billing_account",
        a_key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let b_key_pair = create_key_pair(&ctx).await;
    standard_model::set_belongs_to(
        &ctx,
        "key_pair_belongs_to_billing_account",
        b_key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let visibility_head = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(visibility_head);
    let no_head: Vec<KeyPair> = standard_model::has_many(
        &head_ctx,
        "key_pair_belongs_to_billing_account",
        "key_pairs",
        billing_account.id(),
    )
    .await
    .expect("cannot get key pairs for billing account");
    assert_eq!(no_head.len(), 0, "head relationship should not exist");

    let visibility_change_set = create_visibility_change_set(&change_set);
    let change_set_ctx = ctx.clone_with_new_visibility(visibility_change_set);
    let no_change_set: Vec<KeyPair> = standard_model::has_many(
        &change_set_ctx,
        "key_pair_belongs_to_billing_account",
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

    let key_pairs: Vec<KeyPair> = standard_model::has_many(
        &ctx,
        "key_pair_belongs_to_billing_account",
        "key_pairs",
        billing_account.id(),
    )
    .await
    .expect("cannot get key pair for billing account");
    assert_eq!(&key_pairs, &vec![a_key_pair, b_key_pair]);
}

#[test]
async fn associate_many_to_many(ctx: &DalContext<'_, '_>) {
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(visibility);
    let group = create_group(&ctx).await;
    let user_one = create_user(&ctx).await;
    let user_two = create_user(&ctx).await;
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
}

#[test]
async fn disassociate_many_to_many(ctx: &DalContext<'_, '_>) {
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(visibility);
    let group = create_group(&ctx).await;
    let user_one = create_user(&ctx).await;
    let user_two = create_user(&ctx).await;
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::disassociate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");
}

#[test]
async fn many_to_many(ctx: &DalContext<'_, '_>) {
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(visibility);
    let group_one = create_group(&ctx).await;
    let group_two = create_group(&ctx).await;

    let user_one = create_user(&ctx).await;
    let user_two = create_user(&ctx).await;
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group_one.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group_one.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let right_object_id: Option<&UserId> = None;
    let left_object_id: Option<&GroupId> = None;
    let group_users: Vec<User> = standard_model::many_to_many(
        &ctx,
        "group_many_to_many_users",
        "groups",
        "users",
        Some(group_one.id()),
        right_object_id,
    )
    .await
    .expect("cannot get list of users for group");
    assert_eq!(group_users, vec![user_one.clone(), user_two.clone()]);

    let user_one_groups: Vec<Group> = standard_model::many_to_many(
        &ctx,
        "group_many_to_many_users",
        "groups",
        "users",
        left_object_id,
        Some(user_one.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_one_groups, vec![group_one.clone()]);

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        &ctx,
        "group_many_to_many_users",
        "groups",
        "users",
        left_object_id,
        Some(user_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_two_groups, vec![group_one.clone(), group_two.clone()]);

    standard_model::disassociate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        &ctx,
        "group_many_to_many_users",
        "groups",
        "users",
        left_object_id,
        Some(user_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_two_groups, vec![group_one.clone()]);

    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        &ctx,
        "group_many_to_many_users",
        "groups",
        "users",
        left_object_id,
        Some(user_two.id()),
    )
    .await
    .expect("cannot get list of groups for user");
    assert_eq!(user_two_groups, vec![group_one.clone(), group_two.clone()]);
}

#[test]
async fn associate_many_to_many_no_repeat_entries(ctx: &DalContext<'_, '_>) {
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let ctx = ctx.clone_with_new_visibility(visibility);
    let group = create_group(&ctx).await;
    let user_one = create_user(&ctx).await;
    standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    let result = standard_model::associate_many_to_many(
        &ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await;
    assert!(result.is_err(), "should error");
}

#[test]
async fn find_by_attr(ctx: &DalContext<'_, '_>) {
    let _billing_account = create_billing_account(&ctx).await;
    let change_set = create_change_set(ctx).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    let edit_session_visibility = create_visibility_edit_session(&change_set, &edit_session);
    let mut ctx = ctx.clone_with_new_visibility(edit_session_visibility);
    ctx.update_to_universal_head();

    let schema_one = create_schema(&ctx, &SchemaKind::Concept).await;

    let result: Vec<Schema> =
        standard_model::find_by_attr(&ctx, "schemas", "name", &schema_one.name().to_string())
            .await
            .expect("cannot find the object by name");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], schema_one);

    let schema_two = Schema::new(
        &ctx,
        schema_one.name(),
        schema_one.kind(),
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create another schema with the same name");

    let result: Vec<Schema> =
        standard_model::find_by_attr(&ctx, "schemas", "name", &schema_one.name().to_string())
            .await
            .expect("cannot find the object by name");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], schema_one);
    assert_eq!(result[1], schema_two);
}
