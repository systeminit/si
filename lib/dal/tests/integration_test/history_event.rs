use dal::test_harness::{one_time_setup, TestContext};
use dal::{HistoryActor, HistoryEvent, Tenancy};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let tenancy = Tenancy::new_universal();
    let history_event = HistoryEvent::new(
        &txn,
        &nats,
        "change_set.opened",
        &HistoryActor::SystemInit,
        "change set created",
        &serde_json::json!({}),
        &tenancy,
    )
    .await
    .expect("cannot create a new history event");

    assert_eq!(&history_event.actor, &HistoryActor::SystemInit);
    assert_eq!(&history_event.message, "change set created");
    assert_eq!(&history_event.data, &serde_json::json!({}));
    assert_eq!(&history_event.tenancy, &tenancy);
}
