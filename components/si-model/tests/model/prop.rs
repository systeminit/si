mod string {
    use si_model::PropString;
    use si_model_test::{create_new_schema, one_time_setup, TestContext};

    #[tokio::test]
    async fn new() {
        one_time_setup().await.expect("one time setup failed");
        let ctx = TestContext::init().await;
        let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
        let nats = nats_conn.transaction();
        let mut conn = pg.get().await.expect("cannot connect to pg");
        let txn = conn.transaction().await.expect("cannot create txn");
        let schema = create_new_schema(&txn, &nats).await;
        let prop = PropString::new(&txn, &nats, &schema.id, "poop", "canoe", None)
            .await
            .expect("cannot create new prop");
        assert_eq!(prop.name, "poop");
        assert_eq!(prop.description, "canoe");
    }
}
