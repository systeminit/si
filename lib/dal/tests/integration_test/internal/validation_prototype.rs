use dal::{
    func::backend::validation::FuncBackendValidationArgs, validation::Validation, DalContext, Func,
    StandardModel, ValidationPrototype, ValidationPrototypeContext,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let component_bag = bagger.create_component(ctx, "starfield", "starfield").await;
    let prop = component_bag
        .find_prop(ctx, &["root", "domain", "freestar"])
        .await;

    let func_name = "si:validation".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("error fetching builtin function");
    let func = funcs.pop().expect("missing builtin function si:validation");

    let args = FuncBackendValidationArgs::new(Validation::StringEquals {
        value: Some("".to_string()),
        expected: "amon amarth".to_string(),
    });

    let mut builder = ValidationPrototypeContext::builder();
    builder.set_prop_id(*prop.id());
    let _validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&args).expect("serialization failed"),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("cannot create new attribute prototype");
}

#[test]
async fn find_for_prop(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let component_bag = bagger.create_component(ctx, "starfield", "starfield").await;
    let prop = component_bag
        .find_prop(ctx, &["root", "domain", "freestar"])
        .await;

    let func_name = "si:validation".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs.pop().expect("missing builtin function si:validation");

    let first_args = FuncBackendValidationArgs::new(Validation::StringEquals {
        value: Some("".to_string()),
        expected: "amon amarth".to_string(),
    });

    let mut builder = ValidationPrototypeContext::builder();
    builder.set_prop_id(*prop.id());
    let _first_validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&first_args).expect("Serialization failed"),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("cannot create new attribute prototype");

    let second_args = FuncBackendValidationArgs::new(Validation::StringEquals {
        value: Some("".to_string()),
        expected: "twisty monkey".to_string(),
    });
    let mut builder = ValidationPrototypeContext::builder();
    builder.set_prop_id(*prop.id());
    let _second_validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&second_args).expect("Serialization failed"),
        builder
            .to_context(ctx)
            .await
            .expect("could not convert builder to context"),
    )
    .await
    .expect("cannot create new attribute prototype");

    let validation_results = ValidationPrototype::list_for_prop(ctx, *prop.id())
        .await
        .expect("cannot find values");

    assert_eq!(validation_results.len(), 2);
}
