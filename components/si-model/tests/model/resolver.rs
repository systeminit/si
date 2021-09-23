use si_model::Prop;
use si_model_test::{
    create_change_set, create_custom_node, create_edit_session, create_new_prop_string,
    create_new_prop_string_with_parent, create_new_schema, one_time_setup,
    signup_new_billing_account, TestContext,
};

#[tokio::test]
async fn resolve_attributes() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let schema = create_new_schema(&txn, &nats).await;
    let prop = create_new_prop_string(&txn, &nats, &schema).await;
    let _strange =
        create_new_prop_string_with_parent(&txn, &nats, &schema, &Prop::String(prop)).await;

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
    let change_set = create_change_set(&txn, &nats, &nba).await;
    let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
    let node = create_custom_node(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        &nba,
        &change_set,
        &edit_session,
        &schema.entity_type,
    )
    .await;

    txn.commit().await.expect("transaction fucked up");
    let txn = conn.transaction().await.expect("cannot create txn");

    si_model::resolver::resolve_attributes(&txn, &schema.id, &node.object_id)
        .await
        .expect("cannot resolve attributes");
}

mod resolver {
    use si_model::{Resolver, ResolverBackendKind, ResolverOutputKind};
    use si_model_test::{one_time_setup, TestContext};

    #[tokio::test]
    async fn new() {
        one_time_setup().await.expect("one time setup failed");
        let ctx = TestContext::init().await;
        let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
        let nats = nats_conn.transaction();
        let mut conn = pg.get().await.expect("cannot connect to pg");
        let txn = conn.transaction().await.expect("cannot create txn");

        let resolver = Resolver::new(
            &txn,
            &nats,
            "poop",
            "poop canoe",
            ResolverBackendKind::String,
            ResolverOutputKind::String,
        )
        .await
        .expect("cannot create resolver");

        let fetch = Resolver::get_by_name(&txn, "poop")
            .await
            .expect("cannot get resolver");
        assert_eq!(resolver, fetch);
    }

    #[tokio::test]
    async fn get_by_name() {
        one_time_setup().await.expect("one time setup failed");
        let ctx = TestContext::init().await;
        let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
        let nats = nats_conn.transaction();
        let mut conn = pg.get().await.expect("cannot connect to pg");
        let txn = conn.transaction().await.expect("cannot create txn");

        let resolver = Resolver::new(
            &txn,
            &nats,
            "poop",
            "poop canoe",
            ResolverBackendKind::String,
            ResolverOutputKind::String,
        )
        .await
        .expect("cannot create resolver");
        assert_eq!(resolver.name, "poop");
        assert_eq!(resolver.description, "poop canoe");
        assert_eq!(resolver.backend, ResolverBackendKind::String);
        assert_eq!(resolver.output_kind, ResolverOutputKind::String);
    }
}

mod resolver_binding {
    use si_model::{
        resolver::{ResolverBackendKindBinding, ResolverBackendKindStringBinding},
        Resolver, ResolverBackendKind, ResolverBinding, ResolverOutputKind,
    };
    use si_model_test::{
        create_change_set, create_custom_node, create_edit_session, create_new_prop_string,
        create_new_schema, one_time_setup, signup_new_billing_account, TestContext,
    };

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
        let schema = create_new_schema(&txn, &nats).await;
        let prop = create_new_prop_string(&txn, &nats, &schema).await;
        let node = create_custom_node(
            &pg,
            &txn,
            &nats_conn,
            &nats,
            &veritech,
            &nba,
            &change_set,
            &edit_session,
            &schema.entity_type,
        )
        .await;

        let resolver = Resolver::new(
            &txn,
            &nats,
            "string",
            "string Resolver",
            ResolverBackendKind::String,
            ResolverOutputKind::String,
        )
        .await
        .expect("cannot create resolver");

        let backend_binding =
            ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
                value: String::from("spiritbox"),
            });

        let resolver_binding = ResolverBinding::new(
            &txn,
            &nats,
            &resolver.id,
            backend_binding.clone(),
            schema.id.clone(),
            Some(prop.id.clone()),
            Some(node.object_id.clone()),
            None,
            None,
            None,
        )
        .await
        .expect("cannot create resolver binding");

        assert_eq!(&resolver_binding.entity_id, &Some(node.object_id));
        assert_eq!(&resolver_binding.resolver_id, &resolver.id);
        assert_eq!(&resolver_binding.backend_binding, &backend_binding);
        assert_eq!(resolver_binding.change_set_id, None);
        assert_eq!(resolver_binding.edit_session_id, None);
        assert_eq!(resolver_binding.system_id, None);
    }

    #[tokio::test]
    async fn resolve() {
        one_time_setup().await.expect("one time setup failed");
        let ctx = TestContext::init().await;
        let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
        let nats = nats_conn.transaction();
        let mut conn = pg.get().await.expect("cannot connect to pg");
        let txn = conn.transaction().await.expect("cannot create txn");

        let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;
        let change_set = create_change_set(&txn, &nats, &nba).await;
        let edit_session = create_edit_session(&txn, &nats, &nba, &change_set).await;
        let schema = create_new_schema(&txn, &nats).await;
        let prop = create_new_prop_string(&txn, &nats, &schema).await;
        let node = create_custom_node(
            &pg,
            &txn,
            &nats_conn,
            &nats,
            &veritech,
            &nba,
            &change_set,
            &edit_session,
            &schema.entity_type,
        )
        .await;

        let resolver = Resolver::new(
            &txn,
            &nats,
            "string",
            "string Resolver",
            ResolverBackendKind::String,
            ResolverOutputKind::String,
        )
        .await
        .expect("cannot create resolver");

        let backend_binding =
            ResolverBackendKindBinding::String(ResolverBackendKindStringBinding {
                value: String::from("spiritbox"),
            });

        let resolver_binding = ResolverBinding::new(
            &txn,
            &nats,
            &resolver.id,
            backend_binding.clone(),
            schema.id.clone(),
            Some(prop.id.clone()),
            Some(node.object_id.clone()),
            None,
            None,
            None,
        )
        .await
        .expect("cannot create resolver binding");
        let json_string = resolver_binding
            .resolve()
            .await
            .expect("cannot resolve binding");
        assert_eq!(json_string, Some(serde_json::json!["spiritbox"]));
    }
}
