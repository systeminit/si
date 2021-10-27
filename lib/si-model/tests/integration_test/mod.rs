mod billing_account;
mod change_set;
mod edit_session;
mod history_event;
mod jwt_key;
mod standard_model;
mod tenancy;
mod visibility;

#[macro_export]
macro_rules! test_setup {
    ($ctx:ident, $secret_key:ident, $pg:ident, $pgconn:ident, $pgtxn:ident, $nats_conn:ident, $nats:ident) => {
        si_model::test_harness::one_time_setup()
            .await
            .expect("one time setup failed");
        let $ctx = si_model::test_harness::TestContext::init().await;
        let ($pg, $nats_conn, $secret_key) = $ctx.entries();
        let $nats = $nats_conn.transaction();
        let mut $pgconn = $pg.get().await.expect("cannot connect to pg");
        let $pgtxn = $pgconn.transaction().await.expect("cannot create txn");
    };
}
