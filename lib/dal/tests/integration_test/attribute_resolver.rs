use crate::test_setup;

use dal::{
    attribute_resolver::{AttributeResolverContext, UNSET_ID_VALUE},
    func::{backend::FuncBackendStringArgs, binding::FuncBinding},
    test_harness::{billing_account_signup, create_component_for_schema},
    AttributeResolver, Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, Schema,
    StandardModel, SystemId, Tenancy, Visibility,
};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let veritech = veritech::Client::new(nats_conn.clone());

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
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("gatecreeper".to_string());
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
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");

    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_component_id(*component.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");
}

// TODO: Flesh out the full precedence for attributes, and refactor this test so that it is less
// nuts.
#[tokio::test]
async fn find_value_for_prop_and_component() {
    test_setup!(ctx, secret_key, pg, _conn, txn, nats_conn, nats);
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let veritech = veritech::Client::new(nats_conn.clone());

    let _unset_system_id: SystemId = UNSET_ID_VALUE.into();

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
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("prop".to_string());
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
        .execute(&txn, &nats, veritech.clone())
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        func_binding_return_value.value(),
        Some(&serde_json::json!["prop"])
    );

    let args = FuncBackendStringArgs::new("prop_and_schema".to_string());
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
        .execute(&txn, &nats, veritech.clone())
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_schema_id(*schema.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        func_binding_return_value.value(),
        Some(&serde_json::json!["prop_and_schema"])
    );

    let args = FuncBackendStringArgs::new("prop_and_schema_variant".to_string());
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
        .execute(&txn, &nats, veritech.clone())
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_schema_variant_id(*default_variant.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        func_binding_return_value.value(),
        Some(&serde_json::json!["prop_and_schema_variant"])
    );

    let args = FuncBackendStringArgs::new("prop_and_schema_variant".to_string());
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
        .execute(&txn, &nats, veritech.clone())
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_schema_variant_id(*default_variant.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        func_binding_return_value.value(),
        Some(&serde_json::json!["prop_and_schema_variant"])
    );

    let args = FuncBackendStringArgs::new("prop_and_component".to_string());
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
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");
    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_component_id(*component.id());
    let _attribute_resolver = AttributeResolver::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create attribute resolver");
    let func_binding_return_value = AttributeResolver::find_value_for_prop_and_component(
        &txn,
        &tenancy,
        &visibility,
        *first_prop.id(),
        *component.id(),
        *nba.system.id(),
    )
    .await
    .expect("cannot get return value for prop and component");
    assert_eq!(
        func_binding_return_value.value(),
        Some(&serde_json::json!["prop_and_component"])
    );
}

#[tokio::test]
async fn upsert() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let veritech = veritech::Client::new(nats_conn.clone());

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
        "test:setString",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");

    let args = FuncBackendStringArgs::new("gatecreeper".to_string());
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
        .execute(&txn, &nats, veritech.clone())
        .await
        .expect("failed to execute func binding");

    let mut attribute_resolver_context = AttributeResolverContext::new();
    attribute_resolver_context.set_prop_id(*first_prop.id());
    attribute_resolver_context.set_component_id(*component.id());
    let _first_attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *func_binding.id(),
        attribute_resolver_context.clone(),
    )
    .await
    .expect("cannot create new attribute resolver");

    let second_args = FuncBackendStringArgs::new("bleed from within".to_string());
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
        .execute(&txn, &nats, veritech)
        .await
        .expect("failed to execute func binding");
    let second_attribute_resolver = AttributeResolver::upsert(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *func.id(),
        *second_func_binding.id(),
        attribute_resolver_context,
    )
    .await
    .expect("cannot create new attribute resolver");

    assert_eq!(
        second_attribute_resolver.func_binding_id(),
        *second_func_binding.id()
    );
}
