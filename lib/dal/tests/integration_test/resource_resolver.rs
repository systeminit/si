use dal::DalContext;

use crate::dal::test;
use dal::func::backend::js_resource::FuncBackendJsResourceSyncArgs;
use dal::{
    func::binding::FuncBinding,
    resource_resolver::{ResourceResolverContext, UNSET_ID_VALUE},
    test_harness::create_component_for_schema_variant,
    Func, ResourceResolver, Schema, StandardModel,
};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("No default schema variant found for schema docker_image");

    let component = create_component_for_schema_variant(ctx, schema_variant.id()).await;

    let func_name = "si:resourceSyncHammer".to_owned();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:resourceSyncHammer");

    let args = FuncBackendJsResourceSyncArgs {
        component: component
            .veritech_resource_sync_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component resource_sync view"),
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

    let mut resource_resolver_context = ResourceResolverContext::new();
    resource_resolver_context.set_component_id(*component.id());
    let _resource_esolver = ResourceResolver::new(
        ctx,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resource_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");
}

#[test]
async fn find_for_prototype(ctx: &DalContext<'_, '_>) {
    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("No default schema variant found for schema docker_image");

    let component = create_component_for_schema_variant(ctx, schema_variant.id()).await;

    let func_name = "si:resourceSyncHammer".to_owned();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:resourceSyncHammer");

    let args = FuncBackendJsResourceSyncArgs {
        component: component
            .veritech_resource_sync_component(ctx, UNSET_ID_VALUE.into())
            .await
            .expect("could not create component resource_sync view"),
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

    let mut resolver_context = ResourceResolverContext::new();
    resolver_context.set_component_id(*component.id());
    let created = ResourceResolver::new(
        ctx,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let found_resolver = ResourceResolver::get_for_prototype_and_component(
        ctx,
        &UNSET_ID_VALUE.into(),
        component.id(),
    )
    .await
    .expect("cannot find resolvers");
    let found = found_resolver.expect("found no resource_sync resolvers");
    assert_eq!(created, found);
}
