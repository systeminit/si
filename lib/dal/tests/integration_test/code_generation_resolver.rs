use crate::test_setup;

use crate::dal::test;
use dal::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use dal::{
    code_generation_resolver::{CodeGenerationResolverContext, UNSET_ID_VALUE},
    func::binding::FuncBinding,
    test_harness::{billing_account_signup, create_component_for_schema_variant},
    CodeGenerationResolver, Func, HistoryActor, Schema, StandardModel, Tenancy, Visibility,
};

#[test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        veritech,
        encr_key
    );
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
        .default_variant(
            &txn,
            &tenancy
                .clone_into_read_tenancy(&txn)
                .await
                .expect("unable to generate read tenancy"),
            &visibility,
        )
        .await
        .expect("No default schema variant found for schema docker_image");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let func_name = "si:generateYAML".to_owned();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");

    let args = FuncBackendJsCodeGenerationArgs {
        component: component
            .veritech_code_generation_component(&txn, &tenancy, &visibility, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component code_generation view"),
    };
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");

    let mut code_generation_resolver_context = CodeGenerationResolverContext::new();
    code_generation_resolver_context.set_component_id(*component.id());
    let _code_generation_esolver = CodeGenerationResolver::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        code_generation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");
}

#[test]
async fn find_for_prototype() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech, encr_key);
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
        .default_variant(
            &txn,
            &tenancy
                .clone_into_read_tenancy(&txn)
                .await
                .expect("unable to generate read tenancy"),
            &visibility,
        )
        .await
        .expect("No default schema variant found for schema docker_image");

    let component = create_component_for_schema_variant(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;

    let func_name = "si:generateYAML".to_owned();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");

    let args = FuncBackendJsCodeGenerationArgs {
        component: component
            .veritech_code_generation_component(&txn, &tenancy, &visibility, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component code_generation view"),
    };
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        serde_json::to_value(args.clone()).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");

    let mut resolver_context = CodeGenerationResolverContext::new();
    resolver_context.set_component_id(*component.id());
    let created = CodeGenerationResolver::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let mut found_resolver = CodeGenerationResolver::find_for_prototype_and_component(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        &UNSET_ID_VALUE.into(),
        component.id(),
    )
    .await
    .expect("cannot find resolvers");
    let found = found_resolver
        .pop()
        .expect("found no code_generation resolvers");
    assert_eq!(created, found);
}
