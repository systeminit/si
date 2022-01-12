use crate::test_setup;

use dal::{
    func::{
        backend::FuncBackendStringArgs, binding::FuncBinding,
        binding_return_value::FuncBindingReturnValue,
    },
    test_harness::{create_func, create_func_binding},
    Func, FuncBackendKind, FuncBackendResponseType, HistoryActor, StandardModel, Tenancy,
    Visibility,
};

#[tokio::test]
async fn new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _func = Func::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "poop",
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func");
}

#[tokio::test]
async fn func_binding_new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("floop".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let _func_binding = FuncBinding::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args_json,
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create func binding");
}

#[tokio::test]
async fn func_binding_return_value_new() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("funky".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");

    let func_binding = create_func_binding(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args_json,
        *func.id(),
        *func.backend_kind(),
    )
    .await;

    let _func_binding_return_value = FuncBindingReturnValue::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        Some(serde_json::json!("funky")),
        Some(serde_json::json!("funky")),
        *func.id(),
        *func_binding.id(),
    )
    .await
    .expect("failed to create return value");
}

#[tokio::test]
async fn func_binding_execute() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, _nats_conn, nats);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = serde_json::to_value(FuncBackendStringArgs::new("funky".to_string()))
        .expect("cannot serialize args to json");

    let func_binding = create_func_binding(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        args,
        *func.id(),
        *func.backend_kind(),
    )
    .await;

    let return_value = func_binding
        .execute(&txn, &nats)
        .await
        .expect("failed to execute func binding");
    assert_eq!(return_value.value(), Some(&serde_json::json!["funky"]));
    assert_eq!(
        return_value.unprocessed_value(),
        Some(&serde_json::json!["funky"])
    );
}
