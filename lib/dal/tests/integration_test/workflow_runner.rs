use crate::dal::test;
use dal::func::backend::js_workflow::FuncBackendJsWorkflowArgs;
use dal::workflow_runner::workflow_runner_state::WorkflowRunnerStatus;

use dal::DalContext;
use dal::{
    func::binding::FuncBinding, workflow_prototype::WorkflowPrototypeContext,
    workflow_runner::WorkflowRunnerContext, Func, StandardModel, WorkflowPrototype,
    WorkflowPrototypeId, WorkflowResolverId, WorkflowRunner,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext<'_, '_, '_>) {
    let func_name = "si:poemWorkflow".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let args = FuncBackendJsWorkflowArgs;
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let workflow_runner_context = WorkflowRunnerContext::new();
    let _workflow_runner = WorkflowRunner::new(
        ctx,
        WorkflowPrototypeId::NONE,
        WorkflowResolverId::NONE,
        *func.id(),
        *func_binding.id(),
        workflow_runner_context,
    )
    .await
    .expect("cannot create new workflow runner");
}

#[test]
async fn find_for_prototype(ctx: &DalContext<'_, '_, '_>) {
    let func_name = "si:poemWorkflow".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let args = FuncBackendJsWorkflowArgs;
    let func_binding = FuncBinding::new(
        ctx,
        serde_json::to_value(args.clone()).expect("cannot turn args into json"),
        *func.id(),
        *func.backend_kind(),
    )
    .await
    .expect("cannot create function binding");
    func_binding
        .execute(ctx)
        .await
        .expect("failed to execute func binding");

    let runner_context = WorkflowRunnerContext::new();
    let created = WorkflowRunner::new(
        ctx,
        WorkflowPrototypeId::NONE,
        WorkflowResolverId::NONE,
        *func.id(),
        *func_binding.id(),
        runner_context,
    )
    .await
    .expect("cannot create new workflow runner");

    let mut found_runners = WorkflowRunner::find_for_prototype(
        ctx,
        &WorkflowPrototypeId::NONE,
        WorkflowRunnerContext::default(),
    )
    .await
    .expect("cannot find runners");
    assert_eq!(found_runners.len(), 1);
    let found = found_runners.pop().expect("found no workflow runners");
    assert_eq!(created, found);
}

#[test]
async fn run(ctx: &DalContext<'_, '_, '_>) {
    let name = "si:poemWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));

    let prototype_context = WorkflowPrototypeContext::new();
    let prototype = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        prototype_context,
        "prototype",
    )
    .await
    .expect("cannot create new prototype");
    let (_, state, _) = WorkflowRunner::run(ctx, *prototype.id())
        .await
        .expect("unable to run workflow");
    assert_eq!(state.status(), WorkflowRunnerStatus::Success);
}

#[test]
async fn fail(ctx: &DalContext<'_, '_, '_>) {
    let name = "si:failureWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {}", name));

    let prototype_context = WorkflowPrototypeContext::new();
    let prototype = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        prototype_context,
        "prototype",
    )
    .await
    .expect("cannot create new prototype");

    let (_, state, _) = WorkflowRunner::run(ctx, *prototype.id())
        .await
        .expect("unable to run workflow");
    assert_eq!(
        state.error_message().expect("no error message found"),
        "oopsie!"
    );
    assert_eq!(state.status(), WorkflowRunnerStatus::Failure);
}
