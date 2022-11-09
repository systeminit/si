use dal::{
    confirmation_prototype::ConfirmationPrototypeContext, ConfirmationPrototype, DalContext, Func,
    Schema, StandardModel, SystemId,
};
use dal_test::{test, test_harness::create_component_for_schema};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let func_name = "si:resourceExistsConfirmation";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("unable to find function")
        .pop()
        .expect("unable to find function");
    let context = ConfirmationPrototypeContext::default();
    ConfirmationPrototype::new(ctx, "Do I even exist?", *func.id(), context)
        .await
        .expect("unable to create confirmation prototype");
}

#[test]
async fn find_for_component(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("unable to find schema")
        .pop()
        .expect("unable to find schema");
    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("unable to find default schema variant");
    let component = create_component_for_schema(ctx, schema.id()).await;

    let func_name = "si:resourceExistsConfirmation";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("unable to find function")
        .pop()
        .expect("unable to find function");
    let context = ConfirmationPrototypeContext {
        component_id: *component.id(),
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
        system_id: SystemId::NONE,
    };
    let new_prototype = ConfirmationPrototype::new(ctx, "A nice name", *func.id(), context)
        .await
        .expect("unable to create confirmation prototype");

    let found_prototypes =
        ConfirmationPrototype::list_for_component(ctx, *component.id(), SystemId::NONE)
            .await
            .expect("could not find for context");
    // doesnt find builtins
    assert_eq!(found_prototypes.len(), 1);
    assert_eq!(new_prototype, found_prototypes[0]);
}

#[test]
async fn run(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Key Pair".to_string())
        .await
        .expect("unable to find schema")
        .pop()
        .expect("unable to find schema");
    let component = create_component_for_schema(ctx, schema.id()).await;

    let prototype = ConfirmationPrototype::list_for_component(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("could not find for context")
        .pop()
        .expect("unable to find for context");

    let resolver = prototype
        .run(ctx, *component.id(), SystemId::NONE)
        .await
        .expect("failed to run prototype");
    assert_eq!(resolver.success(), Some(&false));
    assert_eq!(resolver.message(), None);

    let mut recommended_actions = resolver
        .recommended_actions(ctx)
        .await
        .expect("unable to list recommended actions");
    assert_eq!(recommended_actions.len(), 1);

    let action = recommended_actions
        .pop()
        .expect("unable to pop recommended actions");
    assert_eq!(action.name(), "create");
}
