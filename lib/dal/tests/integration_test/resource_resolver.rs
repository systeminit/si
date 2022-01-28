use crate::test_setup;

use dal::func::backend::FuncBackendJsResourceSyncArgs;
use dal::{
    func::binding::FuncBinding,
    resource_resolver::{ResourceResolverContext, UNSET_ID_VALUE},
    test_harness::{billing_account_signup, create_component_for_schema_variant},
    Func, HistoryActor, ResourceResolver, Schema, StandardModel, Tenancy, Visibility,
};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let schema_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("No default schema variant found for schema docker_image");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let func_name = "si:resourceSyncHammer".to_owned();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:resourceSyncHammer");

    let args = FuncBackendJsResourceSyncArgs {
        component: component
            .veritech_resource_sync_component(&txn, &tenancy, &visibility)
            .await
            .expect("could not create component resource_sync view"),
    };
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");

    let mut resource_resolver_context = ResourceResolverContext::new();
    resource_resolver_context.set_component_id(*component.id());
    let _resource_esolver = ResourceResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resource_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");
}

#[tokio::test]
async fn find_for_prototype() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let schema_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("No default schema variant found for schema docker_image");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let func_name = "si:resourceSyncHammer".to_owned();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:resourceSyncHammer");

    let args = FuncBackendJsResourceSyncArgs {
        component: component
            .veritech_resource_sync_component(&txn, &tenancy, &visibility)
            .await
            .expect("could not create component resource_sync view"),
    };
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(args.clone()).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");

    let mut resolver_context = ResourceResolverContext::new();
    resolver_context.set_component_id(*component.id());
    let created = ResourceResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let found_resolver = ResourceResolver::get_for_prototype_and_component(
        &txn,
        &tenancy,
        &visibility,
        &UNSET_ID_VALUE.into(),
        component.id(),
    )
    .await
    .expect("cannot find resolvers");
    let found = found_resolver.expect("found no resource_sync resolvers");
    assert_eq!(created, found);
}
