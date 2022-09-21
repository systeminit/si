use crate::dal::test;
use dal::workflow_runner::workflow_runner_state::WorkflowRunnerStatus;

use dal::DalContext;
use dal::{
    func::binding::FuncBinding, workflow_prototype::WorkflowPrototypeContext,
    workflow_runner::WorkflowRunnerContext, AttributeReadContext, Component, ComponentId,
    ComponentView, Func, Schema, StandardModel, SystemId, WorkflowPrototype, WorkflowPrototypeId,
    WorkflowResolverId, WorkflowRunner,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn new(ctx: &DalContext) {
    let func_name = "si:poemWorkflow".to_string();
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function")
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let args = serde_json::Value::Null;
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
        Vec::new(),
        Vec::new(),
    )
    .await
    .expect("cannot create new workflow runner");
}

#[test]
async fn find_for_prototype(ctx: &DalContext) {
    let func_name = "si:poemWorkflow".to_string();
    let mut funcs = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("Error fetching builtin function");
    let func = funcs
        .pop()
        .expect("Missing builtin function si:poemWorkflow");

    let args = serde_json::Value::Null;
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
        Vec::new(),
        Vec::new(),
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
async fn fail(ctx: &DalContext) {
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

    let (_, state, _, _, _) = WorkflowRunner::run(ctx, 0, *prototype.id(), ComponentId::NONE)
        .await
        .expect("unable to run workflow");
    assert_eq!(
        state.error_message().expect("no error message found"),
        "oopsie!"
    );
    assert_eq!(state.status(), WorkflowRunnerStatus::Failure);
}

#[test]
async fn run(ctx: &DalContext) {
    let title = "Docker Image Resource Refresh";
    let prototype = WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find docker image resource refresh workflow prototype");

    let schema = Schema::find_by_attr(ctx, "name", &"docker_image")
        .await
        .expect("unable to find docker image schema")
        .pop()
        .expect("unable to find docker image");
    let schema_variant = schema
        .default_variant(ctx)
        .await
        .expect("unable to find default schema variant");
    let (component, _) =
        Component::new_for_schema_with_node(ctx, "systeminit/whiskers", schema.id())
            .await
            .expect("cannot create component");

    let context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        system_id: Some(SystemId::NONE),
        ..AttributeReadContext::default()
    };

    let component_view = ComponentView::for_context(ctx, context)
        .await
        .expect("unable to generate component view for docker image component");
    assert_eq!(component_view.resources.len(), 0);

    let (_runner, state, _func_bindings, _, _) =
        WorkflowRunner::run(ctx, 0, *prototype.id(), *component.id())
            .await
            .expect("unable to run workflow runner");
    assert_eq!(state.status(), WorkflowRunnerStatus::Success);

    let component_view = ComponentView::for_context(ctx, context)
        .await
        .expect("unable to generate component view for docker image component");
    assert_eq!(component_view.resources.len(), 1);
}
