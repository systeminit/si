use crate::test_setup;

use dal::func::backend::FuncBackendJsResourceSyncArgs;
use dal::resource_prototype::ResourcePrototypeContext;
use dal::test_harness::find_or_create_production_system;
use dal::{
    resource_prototype::UNSET_ID_VALUE, test_harness::billing_account_signup, Component, Func,
    HistoryActor, ResourcePrototype, Schema, StandardModel, Tenancy, Visibility,
};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech,);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(&txn, &tenancy, &visibility, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        &name,
        schema.id(),
    )
    .await
    .expect("could not create component");

    let func_name = "si:resourceSyncHammer".to_string();
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

    let mut prototype_context = ResourcePrototypeContext::new();
    prototype_context.set_component_id(*component.id());
    let _prototype = ResourcePrototype::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        serde_json::to_value(&args).expect("serialization failed"),
        prototype_context,
    )
    .await
    .expect("cannot create new prototype");
}

#[tokio::test]
async fn find_for_component() {
    // TODO: This test is brittle, because it relies on the behavior of docker_image. I'm okay
    // with that for now, but not for long. If it breaks before we fix it - future person, I'm
    // sorry. ;)

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
    let default_schema_variant_id = schema
        .default_schema_variant_id()
        .expect("cannot get default schema variant id");

    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech,
        &tenancy,
        &visibility,
        &history_actor,
        "silverado",
        schema.id(),
    )
    .await
    .expect("cannot create new component");

    let found_prototype = ResourcePrototype::get_for_component(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *schema.id(),
        *default_schema_variant_id,
        UNSET_ID_VALUE.into(),
    )
    .await
    .expect("could not create component resource_sync view");
    let _found = found_prototype.expect("found no resource_sync prototypes");
}
