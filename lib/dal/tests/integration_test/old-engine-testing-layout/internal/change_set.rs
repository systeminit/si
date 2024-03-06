use dal::{ChangeSet, ChangeSetStatus, DalContext, Visibility};
use dal_test::{helpers::create_change_set, test, DalContextHeadMutRef, DalContextHeadRef};

#[test]
async fn new(DalContextHeadRef(ctx): DalContextHeadRef<'_>) {
    let change_set = ChangeSet::new(
        ctx,
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
    assert_eq!(&change_set.tenancy, ctx.tenancy());
}

#[test]
async fn apply(ctx: &mut DalContext) {
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not perform get by pk")
        .expect("could not get change set");

    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");
    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);

    ctx.update_visibility(Visibility::new_head(false));
}

#[test]
async fn list_open(DalContextHeadMutRef(ctx): DalContextHeadMutRef<'_>) {
    let a_change_set = create_change_set(ctx).await;
    let b_change_set = create_change_set(ctx).await;
    let mut c_change_set = create_change_set(ctx).await;

    let full_list = ChangeSet::list_open(ctx)
        .await
        .expect("cannot get list of open change sets");
    assert_eq!(full_list.len(), 3);
    assert!(
        full_list.iter().any(|f| f.name == a_change_set.name),
        "change set has first entry"
    );
    assert!(
        full_list.iter().any(|f| f.name == b_change_set.name),
        "change set has second entry"
    );
    assert!(
        full_list.iter().any(|f| f.name == c_change_set.name),
        "change set has third entry"
    );
    c_change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");
    let partial_list = ChangeSet::list_open(ctx)
        .await
        .expect("cannot get list of open change sets");
    assert_eq!(partial_list.len(), 2);
    assert!(
        partial_list.iter().any(|f| f.name == a_change_set.name),
        "change set has first entry"
    );
    assert!(
        partial_list.iter().any(|f| f.name == b_change_set.name),
        "change set has second entry"
    );
}

#[test]
async fn get_by_pk(DalContextHeadRef(ctx): DalContextHeadRef<'_>) {
    let change_set = create_change_set(ctx).await;
    let result = ChangeSet::get_by_pk(ctx, &change_set.pk)
        .await
        .expect("cannot get change set by pk")
        .expect("change set pk should exist");
    assert_eq!(&change_set, &result);
}
