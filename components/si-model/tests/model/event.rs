use si_model::{Event, EventKind, EventStatus};
use si_model::test::{one_time_setup, signup_new_billing_account, TestContext};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");
    assert_eq!(&event.message, "I like cheese");
    assert_eq!(&event.kind, &EventKind::EntityAction);
    assert_eq!(
        &event.context,
        &nba.workspace.si_storable.tenant_ids.clone()
    );
    assert_eq!(&event.status, &EventStatus::Running);
    assert_eq!(&event.parent_id, &None);
}

#[tokio::test]
async fn save() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let mut event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let _pre_save_event = event.clone();
    event.message = String::from("I like my butt");
    event
        .save(&pg, &nats_conn)
        .await
        .expect("cannot save event");
    assert_eq!(&event.message, "I like my butt");
}

#[tokio::test]
async fn unknown() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let mut event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let pre_save_event = event.clone();
    event
        .unknown(&pg, &nats_conn)
        .await
        .expect("cannot update event status");
    assert_eq!(
        pre_save_event.status,
        EventStatus::Running,
        "initial state wasn't running"
    );
    assert_eq!(
        event.status,
        EventStatus::Unknown,
        "did not transition to unknown"
    );
}

#[tokio::test]
async fn success() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let mut event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let pre_save_event = event.clone();
    event
        .success(&pg, &nats_conn)
        .await
        .expect("cannot update event status");
    assert_eq!(
        pre_save_event.status,
        EventStatus::Running,
        "initial state wasn't running"
    );
    assert_eq!(
        event.status,
        EventStatus::Success,
        "did not transition to success"
    );
}

#[tokio::test]
async fn error() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let mut event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let pre_save_event = event.clone();
    event
        .error(&pg, &nats_conn)
        .await
        .expect("cannot update event status");
    assert_eq!(
        pre_save_event.status,
        EventStatus::Running,
        "initial state wasn't running"
    );
    assert_eq!(
        event.status,
        EventStatus::Error,
        "did not transition to error"
    );
}

#[tokio::test]
async fn running() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let mut event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let pre_save_event = event.clone();
    event
        .error(&pg, &nats_conn)
        .await
        .expect("cannot update event status");
    assert_eq!(
        pre_save_event.status,
        EventStatus::Running,
        "initial state wasn't running"
    );
    assert_eq!(
        event.status,
        EventStatus::Error,
        "did not transition to error"
    );
    event
        .running(&pg, &nats_conn)
        .await
        .expect("cannot update event status");
    assert_eq!(
        event.status,
        EventStatus::Running,
        "did not transition to running"
    );
}

// Has parent checks for if a given event has a parent in its tree
// with a given parent id.
#[tokio::test]
async fn has_parent() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let event_prime = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let parent_exists = event_prime
        .has_parent(&txn, "bullshit:mcbullshitterton")
        .await
        .expect("cannot get event object");
    assert_eq!(parent_exists, false);

    let event_secondary = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        Some(event_prime.id.clone()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let secondary_parent_exists = event_secondary
        .has_parent(&txn, &event_prime.id)
        .await
        .expect("cannot get event object");
    assert_eq!(secondary_parent_exists, true);

    let event_tertiary = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        Some(event_secondary.id.clone()),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let tertiary_parent_exists = event_tertiary
        .has_parent(&txn, &event_prime.id)
        .await
        .expect("cannot get event object");
    assert_eq!(tertiary_parent_exists, true);
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let txn = conn.transaction().await.expect("cannot create txn");

    let event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    let same_event = Event::get(&txn, &event.id).await.expect("cannot get event");
    assert_eq!(&event, &same_event);
}

#[tokio::test]
async fn log() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let event = Event::new(
        &pg,
        &nats_conn,
        "I like cheese",
        serde_json::json![{}],
        EventKind::EntityAction,
        nba.workspace.si_storable.tenant_ids.clone(),
        None,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event");

    event
        .log(
            &pg,
            &nats_conn,
            si_model::EventLogLevel::Error,
            "super fun!",
            serde_json::json![{}],
        )
        .await
        .expect("cannot create an eventLog");
}
