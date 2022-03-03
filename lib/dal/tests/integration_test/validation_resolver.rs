use crate::test_setup;

use dal::{
    func::{backend::validation::FuncBackendValidateStringValueArgs, binding::FuncBinding},
    test_harness::{billing_account_signup, create_component_for_schema},
    validation_resolver::{ValidationResolverContext, UNSET_ID_VALUE},
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, Schema, StandardModel, SystemId,
    Tenancy, ValidationResolver, Visibility,
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
        veritech,
        encr_key
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

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:validateString",
        FuncBackendKind::ValidateStringValue,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());
    let func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
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

    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");
}

#[tokio::test]
async fn find_for_prototype() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech, encr_key);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
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

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:validateString",
        FuncBackendKind::ValidateStringValue,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let first_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());
    let first_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(first_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    first_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _first_validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *first_func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let second_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "twisty monkey".to_string());
    let second_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(second_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    second_func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");
    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _second_validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *second_func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let validation_results =
        ValidationResolver::find_for_prototype(&txn, &tenancy, &visibility, &UNSET_ID_VALUE.into())
            .await
            .expect("cannot find validators");

    assert_eq!(validation_results.len(), 2);
}

#[tokio::test]
async fn find_values_for_prop_and_component() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech, encr_key);
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

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:validateString",
        FuncBackendKind::ValidateStringValue,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let first_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());
    let first_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(first_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    first_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _first_validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *first_func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let second_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "twisty monkey".to_string());
    let second_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(second_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    second_func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");
    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _second_validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *second_func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let validation_results = ValidationResolver::find_values_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        unset_system_id,
    )
    .await
    .expect("cannot find values");

    assert_eq!(validation_results.len(), 2);
}

#[tokio::test]
async fn find_values_for_prop_and_component_override() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats, veritech, encr_key);
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

    let component = create_component_for_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        schema.id(),
    )
    .await;

    let func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "test:validateString",
        FuncBackendKind::ValidateStringValue,
        FuncBackendResponseType::Validation,
    )
    .await
    .expect("cannot create func");

    let first_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "amon amarth".to_string());
    let first_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(first_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    first_func_binding
        .execute(&txn, &nats, veritech.clone(), encr_key)
        .await
        .expect("failed to execute func binding");
    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _first_validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *first_func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let second_args =
        FuncBackendValidateStringValueArgs::new(Some("".to_string()), "twisty monkey".to_string());
    let second_func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        serde_json::to_value(second_args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    second_func_binding
        .execute(&txn, &nats, veritech, encr_key)
        .await
        .expect("failed to execute func binding");
    let mut validation_resolver_context = ValidationResolverContext::new();
    validation_resolver_context.set_prop_id(*first_prop.id());
    validation_resolver_context.set_component_id(*component.id());
    let _second_validation_resolver = ValidationResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        UNSET_ID_VALUE.into(),
        *func.id(),
        *second_func_binding.id(),
        validation_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    let validation_results = ValidationResolver::find_values_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        unset_system_id,
    )
    .await
    .expect("cannot find values");

    assert_eq!(validation_results.len(), 2);
}
