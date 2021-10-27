use si_model::test::{
    create_change_set, create_edit_session, create_new_prop, create_new_schema, one_time_setup,
    signup_new_billing_account, TestContext,
};
use si_model::{PropKind, PropVariant};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (prop, _default_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let new_variant = PropVariant::new(
        &txn,
        &nats,
        &prop.id,
        "motorhead",
        "motorhead rules",
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create new variant");

    assert_eq!(&new_variant.name, "motorhead");
    assert_eq!(&new_variant.description, "motorhead rules");
    assert_eq!(&new_variant.kind, &PropKind::String);
}

#[tokio::test]
async fn add_to_schema_variant() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let (_prop, prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    prop_variant
        .add_to_schema_variant(&txn, &schema_variant.id, &change_set.id, &edit_session.id)
        .await
        .expect("cannot add prop variant to schema variant");
}

#[tokio::test]
async fn schema_variants() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_schema, first_schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let (_schema, second_schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let (_prop, prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    prop_variant
        .add_to_schema_variant(
            &txn,
            &first_schema_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot add prop variant to schema variant");

    prop_variant
        .add_to_schema_variant(
            &txn,
            &second_schema_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot add prop variant to schema variant");

    let no_head_schema_variants = prop_variant
        .schema_variants(&txn, None, None)
        .await
        .expect("cannot get list of schema variants for prop");
    assert_eq!(no_head_schema_variants, vec![], "emtpy head");

    let no_change_set_schema_variants = prop_variant
        .schema_variants(&txn, Some(&change_set.id), None)
        .await
        .expect("cannot get list of schema variants for prop");
    assert_eq!(no_change_set_schema_variants, vec![], "emtpy change set");

    let edit_session_schema_variants = prop_variant
        .schema_variants(&txn, Some(&change_set.id), Some(&edit_session.id))
        .await
        .expect("cannot get list of schema variants for prop");

    assert_eq!(
        edit_session_schema_variants,
        vec![first_schema_variant.clone(), second_schema_variant.clone()],
        "for edit sessions"
    );

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    let change_set_schema_variants = prop_variant
        .schema_variants(&txn, Some(&change_set.id), None)
        .await
        .expect("cannot get list of schema variants for prop");

    assert_eq!(
        change_set_schema_variants,
        vec![first_schema_variant.clone(), second_schema_variant.clone()],
        "for change set"
    );

    change_set
        .apply(&txn)
        .await
        .expect("cannot apply change set");

    let head_schema_variants = prop_variant
        .schema_variants(&txn, None, None)
        .await
        .expect("cannot get list of schema variants for prop");
    assert_eq!(
        head_schema_variants,
        vec![first_schema_variant, second_schema_variant],
        "for head"
    );
}

#[tokio::test]
async fn remove_from_schema_variant() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_schema, first_schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let (_schema, second_schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let (_prop, prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    prop_variant
        .add_to_schema_variant(
            &txn,
            &first_schema_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot add prop variant to schema variant");

    let edit_session_schema_variants = prop_variant
        .schema_variants(&txn, Some(&change_set.id), Some(&edit_session.id))
        .await
        .expect("cannot get list of schema variants for prop");

    assert_eq!(
        edit_session_schema_variants,
        vec![first_schema_variant.clone(), second_schema_variant.clone()],
        "exist for edit sessions"
    );

    prop_variant
        .remove_from_schema_variant(
            &txn,
            &second_schema_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot remove prop variant from schema variant");

    let shorter_schema_variants = prop_variant
        .schema_variants(&txn, Some(&change_set.id), Some(&edit_session.id))
        .await
        .expect("cannot get list of schema variants for prop");

    assert_eq!(
        shorter_schema_variants,
        vec![first_schema_variant.clone()],
        "removed second schema variant"
    );

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    let change_set_schema_variants = prop_variant
        .schema_variants(&txn, Some(&change_set.id), None)
        .await
        .expect("cannot get list of schema variants for prop for change set");

    assert_eq!(
        change_set_schema_variants,
        vec![first_schema_variant.clone()],
        "removed second schema variant for edit session"
    );

    change_set
        .apply(&txn)
        .await
        .expect("cannot apply change set");

    let head_schema_variants = prop_variant
        .schema_variants(&txn, None, None)
        .await
        .expect("cannot get list of schema variants for prop for head");

    assert_eq!(
        head_schema_variants,
        vec![first_schema_variant.clone()],
        "removed second schema variant for edit session"
    );
}

#[tokio::test]
async fn set_parent() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, child_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let (_prop, parent_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    child_prop_variant
        .add_parent(
            &txn,
            &parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot add child prop variant to parent prop variant");
}

#[tokio::test]
async fn set_parent_not_on_object_or_array() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, child_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let (_prop, parent_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let err = child_prop_variant
        .add_parent(
            &txn,
            &parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await;
    assert!(err.is_err(), "added a parent to a string; not allowed");
}

#[tokio::test]
async fn unset_parent() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, child_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let (_prop, parent_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    child_prop_variant
        .add_parent(
            &txn,
            &parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("set parent failed");

    child_prop_variant
        .remove_parent(
            &txn,
            &parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("could not unset parent");
}

#[tokio::test]
async fn parent() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, child_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let (_prop, parent_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    child_prop_variant
        .add_parent(
            &txn,
            &parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("set parent failed");

    let no_head_parent = child_prop_variant
        .parents(&txn, None, None)
        .await
        .expect("cannot get parent");
    assert!(
        no_head_parent.is_empty(),
        "prop should not have any parents on head"
    );

    let no_change_set_parent = child_prop_variant
        .parents(&txn, Some(&change_set.id), None)
        .await
        .expect("cannot get parent");
    assert!(
        no_change_set_parent.is_empty(),
        "prop should not have any parents on head"
    );

    let edit_session_parents = child_prop_variant
        .parents(&txn, Some(&change_set.id), Some(&edit_session.id))
        .await
        .expect("cannot get parent");

    assert_eq!(&edit_session_parents, &vec![parent_prop_variant.clone()]);

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    let change_set_parents = child_prop_variant
        .parents(&txn, Some(&change_set.id), None)
        .await
        .expect("cannot get parent");

    assert_eq!(&change_set_parents, &vec![parent_prop_variant.clone()]);

    change_set
        .apply(&txn)
        .await
        .expect("cannot apply change set");

    let head_parent = child_prop_variant
        .parents(&txn, None, None)
        .await
        .expect("cannot get parent");

    assert_eq!(&head_parent, &vec![parent_prop_variant]);
}

#[tokio::test]
async fn multiple_parents() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, child_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
        .await;

    let (_prop, parent_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
        .await;

    child_prop_variant
        .add_parent(
            &txn,
            &parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("set parent failed");

    let (_prop, second_parent_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
        .await;

    child_prop_variant
        .add_parent(
            &txn,
            &second_parent_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("set parent failed");

    let edit_session_parents = child_prop_variant
        .parents(&txn, Some(&change_set.id), Some(&edit_session.id))
        .await
        .expect("cannot get parent");

    assert_eq!(&edit_session_parents, &vec![parent_prop_variant, second_parent_prop_variant]);
}


#[tokio::test]
async fn descendants() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, not_in_tree_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    let (_prop, not_in_tree_prop_variant_2) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    not_in_tree_prop_variant
        .add_child(
            &txn,
            &not_in_tree_prop_variant_2.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot add child");

    let (_prop, root_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    not_in_tree_prop_variant_2
        .add_child(
            &txn,
            &root_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("do it");
    dbg!(&root_prop_variant);

    let (_prop, foo) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    foo.add_parent(
        &txn,
        &root_prop_variant.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot set parent");

    let (_prop, foo_bar) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    foo_bar
        .add_parent(&txn, &foo.id, &change_set.id, &edit_session.id)
        .await
        .expect("set parent failed");

    let (_prop, baz) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    baz.add_parent(
        &txn,
        &root_prop_variant.id,
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot set parent");

    let (_prop, bang) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    bang.add_parent(&txn, &baz.id, &change_set.id, &edit_session.id)
        .await
        .expect("cannot set parent");

    let (_prop, boom) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;
    boom.add_parent(&txn, &bang.id, &change_set.id, &edit_session.id)
        .await
        .expect("set parent failed");
    edit_session.save_session(&txn).await.expect("cannot save edit session");
    change_set.apply(&txn).await.expect("cannot apply change set");

    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (_prop, shnizzle) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
        .await;
    shnizzle.add_parent(&txn, &bang.id, &change_set.id, &edit_session.id)
        .await
        .expect("set parent failed much later");
    let d = root_prop_variant.descendants(&txn, None, None).await.expect("cannot get descendants");
    txn.commit().await.expect("should save");

    assert!(false);
}
