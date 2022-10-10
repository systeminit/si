use dal::DalContext;
use dal::{
    confirmation_resolver::ConfirmationResolverContext, test_harness::create_component_for_schema,
    ActionPrototype, ConfirmationPrototype, ConfirmationResolver, FuncBindingId, FuncId, Schema,
    StandardModel, SystemId,
};

use crate::dal::test;

#[test]
async fn new(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("unable to find schema")
        .pop()
        .expect("unable to find schema");
    let component = create_component_for_schema(ctx, schema.id()).await;

    let prototype = ConfirmationPrototype::find_for_component(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find for context")
        .pop()
        .expect("unable to find for context");

    let action = ActionPrototype::find_by_name(
        ctx,
        "create",
        prototype.schema_id(),
        prototype.schema_variant_id(),
        SystemId::NONE,
    )
    .await
    .expect("unable to find action")
    .expect("unable to find action");

    let mut context = ConfirmationResolverContext::new();
    context.set_schema_id(prototype.schema_id());
    context.set_schema_variant_id(prototype.schema_variant_id());

    let resolver = ConfirmationResolver::new(
        ctx,
        *prototype.id(),
        true,
        None,
        vec![action.clone()],
        FuncId::NONE,
        FuncBindingId::NONE,
        context,
    )
    .await
    .expect("unable to create confirmation resolver");
    assert!(resolver.success());
    assert_eq!(resolver.message(), None);
    assert_eq!(
        resolver
            .recommended_actions(ctx)
            .await
            .expect("unable to list recommended actions"),
        vec![action]
    );
}

#[test]
async fn find_for_prototype(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("unable to find schema")
        .pop()
        .expect("unable to find schema");
    let component = create_component_for_schema(ctx, schema.id()).await;

    let prototype = ConfirmationPrototype::find_for_component(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find for context")
        .pop()
        .expect("unable to find for context");

    let mut context = ConfirmationResolverContext::new();
    context.set_schema_id(prototype.schema_id());
    context.set_schema_variant_id(prototype.schema_variant_id());
    assert_eq!(
        ConfirmationResolver::find_for_prototype(ctx, prototype.id(), context.clone())
            .await
            .expect("unable to find for prototype"),
        Vec::new()
    );

    let resolver = prototype
        .run(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("failed to run prototype");
    assert_eq!(
        vec![resolver],
        ConfirmationResolver::find_for_prototype(ctx, prototype.id(), context)
            .await
            .expect("unable to find for prototype")
    );
}
