use crate::dal::test;
use dal::{BillingAccountSignup, ChangeSet, DalContext, Func, WriteTenancy};

use dal::test_harness::{
    create_billing_account, create_billing_account_with_name, create_func, create_group,
    create_key_pair, create_schema, create_user, create_visibility_head,
};
use dal::{
    component::ComponentKind, standard_model, BillingAccount, FuncBackendKind, Group, GroupId,
    KeyPair, Schema, SchemaKind, StandardModel, User, UserId, NO_CHANGE_SET_PK,
};
use itertools::Itertools;

#[test]
async fn get_by_pk(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let retrieved = standard_model::get_by_pk(ctx, "billing_accounts", nba.billing_account.pk())
        .await
        .expect("cannot get billing account by pk");

    assert_eq!(nba.billing_account, retrieved);
}

#[test]
async fn get_by_id(ctx: &DalContext<'_, '_>) {
    let billing_account = create_billing_account_with_name(ctx, "coheed").await;
    let head_visibility = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(head_visibility);

    let no_head: Option<BillingAccount> =
        standard_model::get_by_id(&head_ctx, "billing_accounts", billing_account.id())
            .await
            .expect("could not get billing account by id");

    assert!(no_head.is_none(), "head object exists when it should not");

    let for_change_set: BillingAccount =
        standard_model::get_by_id(ctx, "billing_accounts", billing_account.id())
            .await
            .expect("could not get billing account by id")
            .expect("change set object should exist but it does not");
    assert_eq!(&for_change_set.id(), &billing_account.id());
    assert_eq!(
        &for_change_set.visibility().change_set_pk,
        &billing_account.visibility().change_set_pk
    );

    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .unwrap()
        .unwrap();

    change_set
        .apply(ctx)
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
}

#[test]
async fn list(ctx: &DalContext<'_, '_>) {
    let _coheed_billing_account = create_billing_account_with_name(ctx, "coheed").await;
    let _spiritbox_billing_account = create_billing_account_with_name(ctx, "spiritbox").await;
    let _zeal_billing_account = create_billing_account_with_name(ctx, "zeal and ardor").await;

    let head_visibility = create_visibility_head();
    let head_ctx = ctx.clone_with_new_visibility(head_visibility);

    let no_head: Vec<BillingAccount> = standard_model::list(&head_ctx, "billing_accounts")
        .await
        .expect("could not get billing account by id");
    assert_eq!(
        no_head.len(),
        1,
        "there are no objects beyond the default to list for head"
    );

    let change_set_set: Vec<BillingAccount> = standard_model::list(ctx, "billing_accounts")
        .await
        .expect("could not get billing account by id");
    assert_eq!(
        change_set_set.len(),
        4,
        "there are 5 objects to list for change set"
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
}

#[test]
async fn update(ctx: &mut DalContext<'_, '_>, nba: &BillingAccountSignup) {
    // Guess what--a billing account's tenancy is universal! So let's make sure our DalContext is
    // appropriately set up
    ctx.update_write_tenancy(WriteTenancy::new_universal());

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
        ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
        first_billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    // You cannot replace the existing belongs to relationship by calling it again with a new id
    match standard_model::set_belongs_to(
        ctx,
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
        ctx,
        "key_pair_belongs_to_billing_account",
        key_pair.id(),
        nba.billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    standard_model::unset_belongs_to(ctx, "key_pair_belongs_to_billing_account", key_pair.id())
        .await
        .expect("cannot set billing account for key pair");
}

#[test]
async fn belongs_to(ctx: &DalContext<'_, '_>) {
    let billing_account = create_billing_account_with_name(ctx, "coheed").await;
    let key_pair = create_key_pair(ctx).await;

    standard_model::set_belongs_to(
        ctx,
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

    let has_change_set: Option<BillingAccount> = standard_model::belongs_to(
        ctx,
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

    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .unwrap()
        .unwrap();

    change_set
        .apply(ctx)
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
    let billing_account = create_billing_account_with_name(ctx, "coheed").await;
    let a_key_pair = create_key_pair(ctx).await;
    standard_model::set_belongs_to(
        ctx,
        "key_pair_belongs_to_billing_account",
        a_key_pair.id(),
        billing_account.id(),
    )
    .await
    .expect("cannot set billing account for key pair");

    let b_key_pair = create_key_pair(ctx).await;
    standard_model::set_belongs_to(
        ctx,
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

    let key_pairs: Vec<KeyPair> = standard_model::has_many(
        ctx,
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
    let group = create_group(ctx).await;
    let user_one = create_user(ctx).await;
    let user_two = create_user(ctx).await;
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
}

#[test]
async fn disassociate_many_to_many(ctx: &DalContext<'_, '_>) {
    let group = create_group(ctx).await;
    let user_one = create_user(ctx).await;
    let user_two = create_user(ctx).await;
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::disassociate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");
}

#[test]
async fn many_to_many(ctx: &DalContext<'_, '_>) {
    let group_one = create_group(ctx).await;
    let group_two = create_group(ctx).await;

    let user_one = create_user(ctx).await;
    let user_two = create_user(ctx).await;
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group_one.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group_one.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let right_object_id: Option<&UserId> = None;
    let left_object_id: Option<&GroupId> = None;
    let group_users: Vec<User> = standard_model::many_to_many(
        ctx,
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
        ctx,
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
        ctx,
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
        ctx,
        "group_many_to_many_users",
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot disassociate many to many");

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        ctx,
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
        ctx,
        "group_many_to_many_users",
        group_two.id(),
        user_two.id(),
    )
    .await
    .expect("cannot associate many to many");

    let user_two_groups: Vec<Group> = standard_model::many_to_many(
        ctx,
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
    let group = create_group(ctx).await;
    let user_one = create_user(ctx).await;
    standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await
    .expect("cannot associate many to many");
    let result = standard_model::associate_many_to_many(
        ctx,
        "group_many_to_many_users",
        group.id(),
        user_one.id(),
    )
    .await;
    assert!(result.is_err(), "should error");
}

#[test]
async fn find_by_attr(ctx: &mut DalContext<'_, '_>) {
    let _billing_account = create_billing_account(ctx).await;
    ctx.update_to_universal_head();

    let schema_one = create_schema(ctx, &SchemaKind::Configuration).await;

    let result: Vec<Schema> =
        standard_model::find_by_attr(ctx, "schemas", "name", &schema_one.name().to_string())
            .await
            .expect("cannot find the object by name");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], schema_one);

    let schema_two = Schema::new(
        ctx,
        schema_one.name(),
        schema_one.kind(),
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create another schema with the same name");

    let result: Vec<Schema> =
        standard_model::find_by_attr(ctx, "schemas", "name", &schema_one.name().to_string())
            .await
            .expect("cannot find the object by name");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], schema_one);
    assert_eq!(result[1], schema_two);
}

#[test]
async fn find_by_attr_in(ctx: &mut DalContext<'_, '_>) {
    let _billing_account = create_billing_account(ctx).await;
    ctx.update_to_universal_head();

    // There are some functions in here already but we don't want to rely on
    // them existing for the test to pass
    let first_result: Vec<Func> = standard_model::find_by_attr_in(
        ctx,
        "funcs",
        "backend_kind",
        &[&"JsQualification".to_string(), &"JsAttribute".to_string()],
    )
    .await
    .expect("cannot find objects by backend_kind in slice");

    let mut func_one = create_func(ctx).await;
    func_one
        .set_backend_kind(ctx, FuncBackendKind::JsQualification)
        .await
        .expect("cannot set func backend kind");

    let mut func_two = create_func(ctx).await;
    func_two
        .set_backend_kind(ctx, FuncBackendKind::JsAttribute)
        .await
        .expect("cannot set func backend kind");

    let result: Vec<Func> = dbg!(standard_model::find_by_attr_in(
        ctx,
        "funcs",
        "backend_kind",
        &[
            &FuncBackendKind::JsQualification.as_ref().to_string(),
            &FuncBackendKind::JsAttribute.as_ref().to_string()
        ],
    )
    .await
    .expect("cannot find objects by backend_kind in slice"));

    assert_eq!(2, result.len() - first_result.len());

    assert_eq!(
        Some(&func_one),
        result
            .iter()
            .filter(|&f| f.id() == func_one.id())
            .at_most_one()
            .expect("could not find at most one func")
    );

    assert_eq!(
        Some(&func_two),
        result
            .iter()
            .filter(|&f| f.id() == func_two.id())
            .at_most_one()
            .expect("could not find at most one func")
    );
}
