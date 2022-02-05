mod attribute_resolver;
mod billing_account;
mod capability;
mod change_set;
mod code_generation_prototype;
mod code_generation_resolver;
mod component;
mod edge;
mod edit_session;
mod func;
mod func_execution;
mod group;
mod history_event;
mod jwt_key;
mod key_pair;
mod node;
mod node_menu;
mod node_position;
mod organization;
mod prop;
mod qualification_check;
mod qualification_prototype;
mod qualification_resolver;
mod resource;
mod resource_prototype;
mod resource_resolver;
mod schema;
mod schematic;
mod socket;
mod standard_model;
mod system;
mod tenancy;
mod user;
mod validation_prototype;
mod validation_resolver;
mod visibility;
mod workspace;

#[macro_export]
macro_rules! test_setup {
    (
        $ctx:ident,
        $secret_key:ident,
        $pg:ident,
        $pgconn:ident,
        $pgtxn:ident,
        $nats_conn:ident,
        $nats:ident,
        $veritech:ident $(,)?
    ) => {
        dal::test_harness::one_time_setup()
            .await
            .expect("one time setup failed");
        let $ctx = dal::test_harness::TestContext::init().await;
        let ($pg, $nats_conn, $veritech, $secret_key) = $ctx.entries();
        let $nats = $nats_conn.transaction();
        let mut $pgconn = $pg.get().await.expect("cannot connect to pg");
        let $pgtxn = $pgconn.transaction().await.expect("cannot create txn");
    };
}
