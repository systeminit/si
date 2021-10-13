use si_model::test::{
    create_change_set, create_edit_session, create_new_prop, create_new_schema, one_time_setup,
    signup_new_billing_account, TestContext,
};
use si_model::{PropKind, Schema, SchemaVariant};

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

    let (schema, default_variant) = Schema::new(
        &txn,
        &nats,
        "si",
        "killswitch engage",
        "killswitchEngage",
        "kill switch!",
        &change_set.id,
        &edit_session.id,
        &nba.billing_account.id,
        &nba.organization.id,
        &nba.workspace.id,
    )
    .await
    .expect("cannot create schema");

    let schema_variant = SchemaVariant::new(
        &txn,
        &nats,
        &schema.id,
        "gojira",
        "new gojira type",
        &change_set.id,
        &edit_session.id,
    )
    .await
    .expect("cannot create schema variant");

    assert_eq!(&schema_variant.schema_id, &schema.id);
    assert_eq!(&schema_variant.name, "gojira");
    assert_eq!(&schema_variant.description, "new gojira type");
    assert_eq!(&schema_variant.root_prop_variant_id, &None);
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
    let (_schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let retrieved_schema_variant =
        SchemaVariant::for_edit_session(&txn, &schema_variant.id, &change_set.id, &edit_session.id)
            .await
            .expect("cannot find schema variant for edit session");

    assert_eq!(schema_variant, retrieved_schema_variant);
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
    let (_schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let should_not_work =
        SchemaVariant::for_change_set(&txn, &schema_variant.id, &change_set.id).await;
    assert!(should_not_work.is_err());

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_schema_variant =
        SchemaVariant::for_change_set(&txn, &schema_variant.id, &change_set.id)
            .await
            .expect("cannot find schema for change set");

    assert_eq!(schema_variant, retrieved_schema_variant);
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
    let (_schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let should_not_work = SchemaVariant::for_head(&txn, &schema_variant.id).await;
    assert!(should_not_work.is_err());

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    change_set
        .apply(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_schema_variant = SchemaVariant::for_head(&txn, &schema_variant.id)
        .await
        .expect("cannot find schema for change set");

    assert_eq!(schema_variant, retrieved_schema_variant);
}

#[tokio::test]
async fn set_root_prop_variant_id() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let mut change_set = create_change_set(&txn, &nats, &nba).await;
    let mut edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let (_schema, mut schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let (root_prop, root_prop_variant) = create_new_prop(
        &txn,
        &nats,
        PropKind::Object,
        &nba,
        &change_set,
        &edit_session,
    )
    .await;

    schema_variant
        .set_root_prop_variant_id(
            &txn,
            &root_prop_variant.id,
            &change_set.id,
            &edit_session.id,
        )
        .await
        .expect("cannot add root prop variant to schema variant");

    assert_eq!(
        schema_variant.root_prop_variant_id.as_ref(), Some(&root_prop_variant.id),
        "root prop variant is set on the initial object"
    );

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    change_set
        .apply(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_schema_variant = SchemaVariant::for_head(&txn, &schema_variant.id)
        .await
        .expect("cannot find schema for change set");

    assert_eq!(schema_variant, retrieved_schema_variant);
}
