use dal::{
    code_generation_resolver::{CodeGenerationResolverContext, UNSET_ID_VALUE},
    func::{backend::js_code_generation::FuncBackendJsCodeGenerationArgs, binding::FuncBinding},
    CodeGenerationResolver, DalContext, Func, Schema, StandardModel,
};
use dal_test::{test, test_harness::create_component_for_schema_variant};

#[test]
async fn new(ctx: &DalContext) {
    let name = "Docker Image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("No default schema variant found for schema `Docker Image`");

    let component = create_component_for_schema_variant(ctx, schema_variant.id()).await;

    let func_name = "si:generateYAML".to_owned();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");

    let args = FuncBackendJsCodeGenerationArgs {
        component: component
            .veritech_code_generation_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component code_generation view"),
    };
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let mut code_generation_resolver_context = CodeGenerationResolverContext::new();
    code_generation_resolver_context.set_component_id(*component.id());
    let _code_generation_esolver = CodeGenerationResolver::new(
        ctx,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        code_generation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");
}

#[test]
async fn find_for_prototype(ctx: &DalContext) {
    let name = "Docker Image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("No default schema variant found for schema `Docker Image`");

    let component = create_component_for_schema_variant(ctx, schema_variant.id()).await;

    let func_name = "si:generateYAML".to_owned();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:generateYAML");

    let args = FuncBackendJsCodeGenerationArgs {
        component: component
            .veritech_code_generation_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component code_generation view"),
    };
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args.clone()).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let mut resolver_context = CodeGenerationResolverContext::new();
    resolver_context.set_component_id(*component.id());
    let created = CodeGenerationResolver::new(
        ctx,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let mut found_resolver = CodeGenerationResolver::find_for_prototype_and_component(
        ctx,
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
