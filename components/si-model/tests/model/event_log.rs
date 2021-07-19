use si_model::{EventLog, EventLogLevel, OutputLineStream};
use si_model_test::{create_event, one_time_setup, signup_new_billing_account, TestContext};
use tokio::io::AsyncReadExt;

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

    let event = create_event(&pg, &nats_conn, &nba).await;

    let event_log = EventLog::new(
        &pg,
        &nats_conn,
        "fading slowly",
        serde_json::json![{}],
        EventLogLevel::Fatal,
        event.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event_log");

    assert_eq!(&event_log.message, "fading slowly");
    assert_eq!(&event_log.payload, &serde_json::json![{}]);
    assert_eq!(&event_log.level, &EventLogLevel::Fatal);
}

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

    let event = create_event(&pg, &nats_conn, &nba).await;

    let event_log = EventLog::new(
        &pg,
        &nats_conn,
        "fading slowly",
        serde_json::json![{}],
        EventLogLevel::Fatal,
        event.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event_log");

    assert_eq!(
        event_log
            .has_parent(&txn, &event.id)
            .await
            .expect("cannot check parent"),
        true
    );
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

    let event = create_event(&pg, &nats_conn, &nba).await;

    let og_event_log = EventLog::new(
        &pg,
        &nats_conn,
        "fading slowly",
        serde_json::json![{}],
        EventLogLevel::Fatal,
        event.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event_log");

    let event_log = EventLog::get(&txn, &og_event_log.id)
        .await
        .expect("cannot get event log");
    assert_eq!(og_event_log, event_log);
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

    let event = create_event(&pg, &nats_conn, &nba).await;

    let mut event_log_back_at_it_again_with_the_white_vans = EventLog::new(
        &pg,
        &nats_conn,
        "fading slowly",
        serde_json::json![{}],
        EventLogLevel::Fatal,
        event.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event_log");

    event_log_back_at_it_again_with_the_white_vans.level = EventLogLevel::Info;
    event_log_back_at_it_again_with_the_white_vans.message = "damn daniel".into();
    event_log_back_at_it_again_with_the_white_vans.payload = serde_json::json![{"cool":"shoes"}];
    event_log_back_at_it_again_with_the_white_vans
        .save(&pg, &nats_conn)
        .await
        .expect("cannot save event log");

    assert_eq!(
        &event_log_back_at_it_again_with_the_white_vans.level,
        &EventLogLevel::Info
    );
    assert_eq!(
        &event_log_back_at_it_again_with_the_white_vans.message,
        "damn daniel"
    );
    assert_eq!(
        &event_log_back_at_it_again_with_the_white_vans.payload,
        &serde_json::json![{"cool":"shoes"}]
    );
}

#[tokio::test]
async fn output_line() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    txn.commit()
        .await
        .expect("failed to commit the new billing account");

    let _txn = conn.transaction().await.expect("cannot create txn");

    let event = create_event(&pg, &nats_conn, &nba).await;

    let mut event_log = EventLog::new(
        &pg,
        &nats_conn,
        "running kubectl",
        serde_json::json![{}],
        EventLogLevel::Info,
        event.id.clone(),
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create event_log");

    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::Stdout,
            "hey stdout",
            false,
        )
        .await
        .expect("cannot create output line");
    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::All,
            "hey all",
            false,
        )
        .await
        .expect("cannot create output line");
    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::Stderr,
            "hey stderr",
            false,
        )
        .await
        .expect("cannot create output line");
    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::Stderr,
            "",
            true,
        )
        .await
        .expect("cannot create output line");
    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::Stdout,
            "okay, done stdout",
            false,
        )
        .await
        .expect("cannot create output line");
    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::Stdout,
            "",
            true,
        )
        .await
        .expect("cannot create output line");
    event_log
        .output_line(
            &pg,
            &nats_conn,
            &event_log_fs,
            OutputLineStream::All,
            "",
            true,
        )
        .await
        .expect("cannot create output line");

    let mut stdout = String::new();
    event_log_fs
        .get_read_handle(&event_log.id, &OutputLineStream::Stdout)
        .await
        .expect("can't get read handle")
        .read_to_string(&mut stdout)
        .await
        .expect("couldn't write to stdout string");

    let mut stderr = String::new();
    event_log_fs
        .get_read_handle(&event_log.id, &OutputLineStream::Stderr)
        .await
        .expect("can't get read handle")
        .read_to_string(&mut stderr)
        .await
        .expect("couldn't write to stderr string");

    let mut all = String::new();
    event_log_fs
        .get_read_handle(&event_log.id, &OutputLineStream::All)
        .await
        .expect("can't get read handle")
        .read_to_string(&mut all)
        .await
        .expect("couldn't write to all string");

    assert_eq!(stdout, "hey stdout\nokay, done stdout\n");
    assert_eq!(stderr, "hey stderr\n");
    assert_eq!(all, "hey all\n");
}
