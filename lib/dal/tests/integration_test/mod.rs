mod billing_account;
mod capability;
mod change_set;
mod component;
mod edge;
mod edit_session;
mod group;
mod history_event;
mod jwt_key;
mod key_pair;
mod node;
mod node_menu;
mod node_position;
mod organization;
mod qualification_check;
mod schema;
mod socket;
mod standard_model;
mod system;
mod tenancy;
mod user;
mod visibility;
mod workspace;

#[macro_export]
macro_rules! test_setup {
    ($ctx:ident, $secret_key:ident, $pg:ident, $pgconn:ident, $pgtxn:ident, $nats_conn:ident, $nats:ident) => {
        dal::test_harness::one_time_setup()
            .await
            .expect("one time setup failed");
        let $ctx = dal::test_harness::TestContext::init().await;
        let ($pg, $nats_conn, $secret_key) = $ctx.entries();
        let $nats = $nats_conn.transaction();
        let mut $pgconn = $pg.get().await.expect("cannot connect to pg");
        let $pgtxn = $pgconn.transaction().await.expect("cannot create txn");
    };
}
