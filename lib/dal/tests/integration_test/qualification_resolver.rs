use dal::{
    func::{backend::js_qualification::FuncBackendJsQualificationArgs, binding::FuncBinding},
    qualification_resolver::{QualificationResolverContext, UNSET_ID_VALUE},
    DalContext, Func, QualificationResolver, Schema, StandardModel,
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
        .expect("No default schema variant found for schema Docker Image");

    let component = create_component_for_schema_variant(ctx, schema_variant.id()).await;

    let func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:qualificationDockerImageNameInspect");

    let args = FuncBackendJsQualificationArgs {
        component: component
            .veritech_qualification_check_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component qualification view"),
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

    let mut qualification_resolver_context = QualificationResolverContext::new();
    qualification_resolver_context.set_component_id(*component.id());
    let _qualification_resolver = QualificationResolver::new(
        ctx,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        qualification_resolver_context,
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

    let func_name = "si:qualificationDockerImageNameInspect".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:qualificationDockerImageNameInspect");

    let args = FuncBackendJsQualificationArgs {
        component: component
            .veritech_qualification_check_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component qualification view"),
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

    let mut resolver_context = QualificationResolverContext::new();
    resolver_context.set_component_id(*component.id());
    let created = QualificationResolver::new(
        ctx,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let mut found_resolvers = QualificationResolver::find_for_prototype_and_component(
        ctx,
        &UNSET_ID_VALUE.into(),
        component.id(),
    )
    .await
    .expect("cannot find resolvers");
    assert_eq!(found_resolvers.len(), 1);
    let found = found_resolvers
        .pop()
        .expect("found no qualification resolvers");
    assert_eq!(created, found);
}
