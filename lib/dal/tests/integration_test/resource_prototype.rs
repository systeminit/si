use dal::DalContext;

use crate::dal::test;
use dal::func::backend::js_resource::FuncBackendJsResourceSyncArgs;
use dal::resource_prototype::ResourcePrototypeContext;
use dal::{
    resource_prototype::UNSET_ID_VALUE, Component, Func, ResourcePrototype, Schema, StandardModel,
};

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let (component, _node, _) = Component::new_for_schema_with_node(ctx, &name, schema.id())
        .await
        .expect("could not create component");

    let func_name = "si:resourceSyncHammer".to_string();
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

    let mut prototype_context = ResourcePrototypeContext::new();
    prototype_context.set_component_id(*component.id());
    let _prototype = ResourcePrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&args).expect("serialization failed"),
        prototype_context,
    )
    .await
    .expect("cannot create new prototype");
}

#[test]
async fn find_for_component(ctx: &DalContext<'_, '_>) {
    // TODO: This test is brittle, because it relies on the behavior of docker_image. I'm okay
    // with that for now, but not for long. If it breaks before we fix it - future person, I'm
    // sorry. ;)

    let name = "docker_image".to_string();
    let schema = Schema::find_by_attr(ctx, "name", &name)
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");
    let default_schema_variant_id = schema
        .default_schema_variant_id()
        .expect("cannot get default schema variant id");

    let (component, _node, _) = Component::new_for_schema_with_node(ctx, "silverado", schema.id())
        .await
        .expect("cannot create new component");

    let found_prototype = ResourcePrototype::get_for_component(
        ctx,
        *component.id(),
        *schema.id(),
        *default_schema_variant_id,
        UNSET_ID_VALUE.into(),
    )
    .await
    .expect("could not create component resource_sync view");
    let _found = found_prototype.expect("found no resource_sync prototypes");
}
