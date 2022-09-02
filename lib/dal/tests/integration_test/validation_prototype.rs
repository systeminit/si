use dal::{BillingAccountSignup, DalContext};

use crate::dal::test;
use dal::{
    func::backend::validation::FuncBackendValidateStringValueArgs,
    validation_prototype::{ValidationPrototypeContext, UNSET_ID_VALUE},
    Func, Schema, StandardModel, SystemId, ValidationPrototype,
};

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let default_variant = schema
        .default_variant(ctx)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(ctx)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:validateStringEquals");

    let args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());

    let mut validation_prototype_context = ValidationPrototypeContext::new();
    validation_prototype_context.set_prop_id(*first_prop.id());
    let _validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&args).expect("Serialization failed"),
        validation_prototype_context,
    )
    .await
    .expect("cannot create new attribute prototype");
}

#[test]
async fn find_for_prop(ctx: &DalContext<'_, '_, '_>, _nba: &BillingAccountSignup) {
    let unset_system_id: SystemId = UNSET_ID_VALUE.into();

    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image")
        .pop()
        .expect("no docker image found");

    let default_variant = schema
        .default_variant(ctx)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(ctx)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:validateStringEquals");

    let first_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());

    let mut validation_prototype_context = ValidationPrototypeContext::new();
    validation_prototype_context.set_prop_id(*first_prop.id());
    let _first_validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&first_args).expect("Serialization failed"),
        validation_prototype_context,
    )
    .await
    .expect("cannot create new attribute prototype");

    let second_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "twisty monkey".to_string());
    let mut validation_prototype_context = ValidationPrototypeContext::new();
    validation_prototype_context.set_prop_id(*first_prop.id());
    let _second_validation_prototype = ValidationPrototype::new(
        ctx,
        *func.id(),
        serde_json::to_value(&second_args).expect("Serialization failed"),
        validation_prototype_context,
    )
    .await
    .expect("cannot create new attribute prototype");

    let validation_results =
        ValidationPrototype::find_for_prop(ctx, *first_prop.id(), unset_system_id)
            .await
            .expect("cannot find values");

    assert_eq!(validation_results.len(), 2);
}
