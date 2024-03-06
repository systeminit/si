use dal::{
    func::{
        backend::string::FuncBackendStringArgs,
        execution::{FuncExecution, FuncExecutionState},
    },
    DalContext, FuncBinding, StandardModel,
};
use dal_test::{
    test,
    test_harness::{create_func, create_func_binding},
};
use veritech_client::OutputStream;

#[test]
async fn new(ctx: &DalContext) {
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let func_binding = create_func_binding(ctx, args_json, *func.id(), *func.backend_kind()).await;
    let execution = FuncExecution::new(ctx, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");
    assert_eq!(execution.state(), FuncExecutionState::Start);
}

#[test]
async fn set_state(ctx: &DalContext) {
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let func_binding = create_func_binding(ctx, args_json, *func.id(), *func.backend_kind()).await;
    let mut execution = FuncExecution::new(ctx, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");
    assert_eq!(execution.state(), FuncExecutionState::Start);
    execution
        .set_state(ctx, FuncExecutionState::Dispatch)
        .await
        .expect("cannot set state");
    assert_eq!(execution.state(), FuncExecutionState::Dispatch);
}

#[test]
async fn set_output_stream(ctx: &DalContext) {
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());
    let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
    let func_binding = create_func_binding(ctx, args_json, *func.id(), *func.backend_kind()).await;
    let mut execution = FuncExecution::new(ctx, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");

    execution
        .set_output_stream(
            ctx,
            vec![
                (OutputStream {
                    stream: "stdout".to_string(),
                    execution_id: "foo".to_string(),
                    level: "info".to_string(),
                    group: None,
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

#[test]
async fn process_return_value(ctx: &DalContext) {
    let func = create_func(ctx).await;
    let args = FuncBackendStringArgs::new("slayer".to_string());

    let (func_binding, func_binding_return_value) = FuncBinding::create_and_execute(
        ctx,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        vec![],
    )
    .await
    .expect("failed to execute func binding");

    let mut execution = FuncExecution::new(ctx, &func, &func_binding)
        .await
        .expect("cannot create a new func execution");

    execution
        .process_return_value(ctx, &func_binding_return_value)
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

// FIXME(nick,fletcher): re-add test once upsert is added.
// #[test]
// async fn execution_upserts_return_value() {
//
//     let tenancy = Tenancy::new_universal();
//     let visibility = create_visibility_head();
//     let history_actor = HistoryActor::SystemInit;
//
//     let func = create_func(ctx).await;
//     let args = FuncBackendStringArgs::new("slayer".to_string());
//     let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
//     let func_binding = create_func_binding(
//         &txn,
//         &nats,
//         &tenancy,
//         &visibility,
//         &history_actor,
//         args_json,
//         *func.id(),
//         *func.backend_kind(),
//     )
//         .await;
//     let func_binding_return_value = func_binding
//         .execute(&txn, &nats, veritech)
//         .await
//         .expect("cannot execute binding");
//
//     let _execution1 = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
//         .await
//         .expect("cannot create a new func execution");
//
//     let  _execution2 = FuncExecution::new(&txn, &nats, &tenancy, &func, &func_binding)
//         .await
//         .expect("cannot create a new func execution");
//
// }
