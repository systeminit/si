use si_model::Schema;
use si_model::test::{one_time_setup, TestContext};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let schema = Schema::new(
        &txn,
        &nats,
        "Test Schema",
        "testSchema",
        "killswitch engage testing",
    )
    .await
    .expect("cannot create schema");
    assert_eq!(schema.name, "Test Schema");
    assert_eq!(schema.entity_type, "testSchema");
    assert_eq!(schema.description, "killswitch engage testing");
}
