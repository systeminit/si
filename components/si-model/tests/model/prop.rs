use si_model::test::{
    create_change_set, create_edit_session, create_new_prop, create_new_schema, one_time_setup,
    signup_new_billing_account, TestContext,
};
use si_model::{Prop, PropKind};

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

    let (prop, default_variant) = Prop::new(
        &txn,
        &nats,
        "default",
        "poop",
        "poop canoe",
        PropKind::String,
        &change_set.id,
        &edit_session.id,
        &nba.billing_account.id,
        &nba.organization.id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create new prop");

    assert_eq!(prop.name, "poop");
    assert_eq!(prop.description, "poop canoe");

    assert_eq!(default_variant.name, "default");
    assert_eq!(default_variant.description, "default");
    assert_eq!(default_variant.kind, PropKind::String);
}

#[tokio::test]
async fn for_edit_session() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let (prop, _prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let retrieved_prop = Prop::for_edit_session(&txn, &prop.id, &change_set.id, &edit_session.id)
        .await
        .expect("cannot find prop for edit session");

    assert_eq!(prop, retrieved_prop);
}

#[tokio::test]
async fn for_change_set() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let (prop, _prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let should_not_work = Prop::for_change_set(&txn, &prop.id, &change_set.id).await;
    assert!(should_not_work.is_err());

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_prop = Prop::for_change_set(&txn, &prop.id, &change_set.id)
        .await
        .expect("cannot find prop for change set");

    assert_eq!(prop, retrieved_prop);
}

#[tokio::test]
async fn for_head() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;

    let (prop, _prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::String,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    let should_not_work = Prop::for_head(&txn, &prop.id).await;
    assert!(should_not_work.is_err());

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    change_set
        .apply(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_prop = Prop::for_head(&txn, &prop.id)
        .await
        .expect("cannot find prop for change set");

    assert_eq!(prop, retrieved_prop);
}
