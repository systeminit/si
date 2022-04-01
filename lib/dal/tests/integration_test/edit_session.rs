use dal::{
    test::{
        helpers::{
            create_change_set, create_change_set_and_edit_session, create_edit_session,
            create_group,
        },
        DalContextHeadMutRef, DalContextHeadRef,
    },
    BillingAccountId, EditSession, EditSessionStatus, Group, StandardModel, Visibility,
    NO_EDIT_SESSION_PK,
};

use crate::dal::test;

#[test]
async fn new(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>, bid: BillingAccountId) {
    let change_set = create_change_set(ctx, bid).await;

    let _edit_session = EditSession::new(ctx, &change_set.pk, "whatever", None)
        .await
        .expect("cannot create edit session");
}

#[test]
async fn save(DalContextHeadMutRef(ctx): DalContextHeadMutRef<'_, '_, '_>, bid: BillingAccountId) {
    let change_set = create_change_set(ctx, bid).await;
    let mut edit_session = create_edit_session(ctx, &change_set).await;

    ctx.update_visibility(Visibility::new_edit_session(
        change_set.pk,
        edit_session.pk,
        false,
    ));

    let group = create_group(ctx).await;

    edit_session
        .save(ctx)
        .await
        .expect("cannot save edit session");

    assert_eq!(&edit_session.status, &EditSessionStatus::Saved);

    ctx.update_visibility(Visibility::new_change_set(change_set.pk, false));

    let change_set_group = Group::get_by_id(ctx, group.id())
        .await
        .expect("cannot get group post edit session save")
        .expect("group not present in change set");

    assert_eq!(group.id(), change_set_group.id());
    assert_ne!(group.pk(), change_set_group.pk());
    assert_eq!(group.name(), change_set_group.name());
    assert_eq!(
        change_set_group.visibility().edit_session_pk,
        NO_EDIT_SESSION_PK
    );
    assert_eq!(
        group.visibility().change_set_pk,
        change_set_group.visibility().change_set_pk
    );
}

#[test]
async fn get_by_pk(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>, bid: BillingAccountId) {
    let (_change_set, edit_session) = create_change_set_and_edit_session(ctx, bid).await;

    let result = EditSession::get_by_pk(ctx, &edit_session.pk)
        .await
        .expect("cannot get edit session by pk")
        .expect("edit session pk should exist");
    assert_eq!(&edit_session, &result);
}

#[test]
async fn cancel(DalContextHeadRef(ctx): DalContextHeadRef<'_, '_, '_>, bid: BillingAccountId) {
    let (_change_set, mut edit_session) = create_change_set_and_edit_session(ctx, bid).await;

    edit_session
        .cancel(ctx)
        .await
        .expect("cannot cancel edit session");
    assert_eq!(&edit_session.status, &EditSessionStatus::Canceled);
}
