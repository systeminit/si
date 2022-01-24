use crate::test_setup;
use dal::func::execution::FuncExecutionState;
use dal::{func::execution::FuncExecution, StandardModel};

use dal::{
    func::backend::FuncBackendStringArgs,
    test_harness::{create_func, create_func_binding, create_visibility_head},
    HistoryActor, Tenancy,
};
use veritech::OutputStream;

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = create_visibility_head();
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
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
    let execution = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");
    assert_eq!(execution.state(), FuncExecutionState::Start);
}

#[tokio::test]
async fn set_state() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = create_visibility_head();
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
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
    let mut execution = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");
    assert_eq!(execution.state(), FuncExecutionState::Start);
    execution
        .set_state(&txn, &nats, FuncExecutionState::Dispatch)
        .await
        .expect("cannot set state");
    assert_eq!(execution.state(), FuncExecutionState::Dispatch);
}

#[tokio::test]
async fn set_output_stream() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = create_visibility_head();
    let history_actor = HistoryActor::SystemInit;
    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
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
    let mut execution = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");

    execution
        .set_output_stream(
            &txn,
            &nats,
            vec![
                (OutputStream {
                    stream: "stdout".to_string(),
                    execution_id: "foo".to_string(),
                    level: "info".to_string(),
                    group: None,
                    data: None,
                    message: "worm shepherd".to_string(),
                    timestamp: 1865,
                }),
            ],
        )
        .await
        .expect("cannot set output stream");
    let output_stream = execution.output_stream().expect("has an output stream");
    assert_eq!(output_stream.len(), 1);
}

#[tokio::test]
async fn process_return_value() {
    test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = create_visibility_head();
    let history_actor = HistoryActor::SystemInit;

    let func = create_func(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
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
    let func_binding_return_value = func_binding
        .execute(&txn, &nats, veritech)
        .await
        .expect("cannot execute binding");

    let mut execution = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");

    execution
        .process_return_value(&txn, &nats, &func_binding_return_value)
        .await
        .expect("cannot process return value");
    assert_eq!(
        execution.func_binding_return_value_id(),
        Some(*func_binding_return_value.id())
    );
    assert_eq!(execution.value(), func_binding_return_value.value(),);
    assert_eq!(
        execution.unprocessed_value(),
        func_binding_return_value.unprocessed_value(),
    );
}
