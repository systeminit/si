mod action_prototype;
mod attribute;
mod builtins;
mod change_set;
mod component;
mod diagram;
mod edge;
mod func;
mod func_execution;
mod graph;
mod history_event;
mod jwt_key;
mod key_pair;
mod node;
mod node_menu;
mod pkg;
mod prop;
mod property_editor;
mod provider;
mod schema;
mod secret;
mod socket;
mod standard_model;
mod status_update;
mod tenancy;
mod user;
mod validation_prototype;
mod validation_resolver;
mod visibility;
mod workflow;
mod workflow_prototype;
mod workflow_resolver;
mod workflow_runner;
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
        $veritech:ident,
        $encryption_key:ident $(,)?
    ) => {
        let test_context = ::dal_test::TestContext::global().await;
        let nats_subject_prefix = ::dal_test::nats_subject_prefix();
        let services_context = test_context
            .create_services_context(&nats_subject_prefix)
            .await;

        // Run a Veritech server instance for each test
        let veritech_server = ::dal_test::veritech_server_for_uds_cyclone(
            test_context.nats_config().clone(),
            format!("{nats_subject_prefix}.veritech"),
        )
        .await;
        ::tokio::spawn(veritech_server.run());

        // Phase out usage of this variable--the assert is to consume/use the variable to avoid
        // Rust warnings for every usage of this macro
        let $ctx = true;
        assert!($ctx);

        let $secret_key = test_context.jwt_secret_key();
        let $pg = services_context.pg_pool();
        let mut $pgconn = $pg
            .get()
            .await
            .expect("failed to get a pg connection from the pool");
        let $pgtxn = $pgconn.transaction().await.expect("cannot create pg txn");
        let $nats_conn = services_context.nats_conn();
        let $nats = $nats_conn.transaction();
        let $veritech = services_context.veritech().clone();
        let $encryption_key = services_context.encryption_key();
    };
}
