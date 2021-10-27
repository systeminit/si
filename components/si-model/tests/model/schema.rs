use si_model::test::{
    create_change_set, create_edit_session, create_new_schema, one_time_setup,
    signup_new_billing_account, TestContext,
};
use si_model::{Schema, SchemaVariant};

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
    assert_eq!(schema.name, "killswitch engage");
    assert_eq!(schema.entity_type, "killswitchEngage");
    assert_eq!(schema.description, "kill switch!");
    assert_eq!(schema.namespace, "si");
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
    let (schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let retrieved_schema =
        Schema::for_edit_session(&txn, &schema.id, &change_set.id, &edit_session.id)
            .await
            .expect("cannot find schema for edit session");

    assert_eq!(schema, retrieved_schema);
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
    let (schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let should_not_work = Schema::for_change_set(&txn, &schema.id, &change_set.id).await;
    assert!(should_not_work.is_err());

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_schema = Schema::for_change_set(&txn, &schema.id, &change_set.id)
        .await
        .expect("cannot find schema for change set");

    assert_eq!(schema, retrieved_schema);
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
    let (schema, schema_variant) =
        create_new_schema(&txn, &nats, &nba, &change_set, &edit_session).await;

    let should_not_work = Schema::for_head(&txn, &schema.id).await;
    assert!(should_not_work.is_err());

    edit_session
        .save_session(&txn)
        .await
        .expect("cannot save edit session");

    change_set
        .apply(&txn)
        .await
        .expect("cannot save edit session");

    let retrieved_schema = Schema::for_head(&txn, &schema.id)
        .await
        .expect("cannot find schema for change set");

    assert_eq!(schema, retrieved_schema);
}
