use crate::test_setup;

use dal::{
    attribute_resolver_context::AttributeResolverContext,
    attribute_value::AttributeValue,
    func::{
        backend::string::FuncBackendStringArgs, binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue, execution::FuncExecution,
    },
    test_harness::{create_func, create_schema, create_schema_variant},
    HistoryActor, SchemaKind, StandardModel, Tenancy, Visibility,
};

use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};

#[tokio::test]
async fn new() {}

#[tokio::test]
async fn update_proxied_value() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("".to_string());
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
    .expect("cannot create func binding");

    let execution = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
        .await
        .expect("cannot create func execution");

    let func_binding_return_value = FuncBindingReturnValue::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        Some(serde_json::json!("")),
        Some(serde_json::json!("")),
        *func.id(),
        *func_binding.id(),
        execution.pk(),
    )
    .await
    .expect("cannot create new fbrv");

    let mut schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot associate variant with schema");
    schema
        .set_default_schema_variant_id(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            Some(*schema_variant.id()),
        )
        .await
        .expect("cannot set default schema variant");

    let mut original_context = AttributeResolverContext::new();
    original_context.set_schema_id(*schema.id());

    let original_value = AttributeValue::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        Some(*func_binding_return_value.id()),
        original_context,
        None,
    )
    .await
    .expect("cannot create new attribute value");
}
