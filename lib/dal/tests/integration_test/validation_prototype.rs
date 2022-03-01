use crate::test_setup;

use dal::{
    func::backend::validation::FuncBackendValidateStringValueArgs,
    test_harness::billing_account_signup,
    validation_prototype::{ValidationPrototypeContext, UNSET_ID_VALUE},
    Func, HistoryActor, Schema, StandardModel, SystemId, Tenancy, ValidationPrototype, Visibility,
};

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image")
    .pop()
    .expect("no docker image found");

    let default_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
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
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        serde_json::to_value(&args).expect("Serialization failed"),
        validation_prototype_context,
    )
    .await
    .expect("cannot create new attribute prototype");
}

#[tokio::test]
async fn find_for_prop() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let unset_system_id: SystemId = UNSET_ID_VALUE.into();

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image")
    .pop()
    .expect("no docker image found");

    let default_variant = schema
        .default_variant(&txn, &tenancy, &visibility)
        .await
        .expect("cannot find default variant");

    let first_prop = default_variant
        .props(&txn, &visibility)
        .await
        .expect("cannot get props")
        .pop()
        .expect("no prop found");

    let func_name = "si:validateStringEquals".to_string();
    let mut funcs = Func::find_by_attr(&txn, &tenancy, &visibility, "name", &func_name)
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
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
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
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        serde_json::to_value(&second_args).expect("Serialization failed"),
        validation_prototype_context,
    )
    .await
    .expect("cannot create new attribute prototype");

    let validation_results = ValidationPrototype::find_for_prop(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        unset_system_id,
    )
    .await
    .expect("cannot find values");

    assert_eq!(validation_results.len(), 2);
}
