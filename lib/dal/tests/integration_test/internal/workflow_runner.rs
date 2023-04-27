use dal::{
    func::binding::FuncBinding,
    workflow_prototype::WorkflowPrototypeContext,
    workflow_runner::{workflow_runner_state::WorkflowRunnerStatus, WorkflowRunnerContext},
    ChangeSet, ChangeSetStatus, Component, DalContext, DependentValuesUpdate, Func, RootPropChild,
    Schema, StandardModel, Visibility, WorkflowPrototype, WorkflowPrototypeId, WorkflowResolverId,
    WorkflowRunner,
};
use dal_test::{test, test_harness::create_component_and_schema};
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
        &[],
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
        &[],
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
async fn fail(ctx: &mut DalContext) {
    let name = "si:failureWorkflow";
    let func = Func::find_by_attr(ctx, "name", &name)
        .await
        .expect("unable to find func")
        .pop()
        .unwrap_or_else(|| panic!("function not found: {name}"));

    let component = create_component_and_schema(ctx).await;
    let schema = component
        .schema(ctx)
        .await
        .expect("error while accessing schema for component")
        .expect("no schema for component");
    let schema_variant = component
        .schema_variant(ctx)
        .await
        .expect("error while accessing schema variant for component")
        .expect("no schema variant for component");

    let mut prototype_context = WorkflowPrototypeContext::new();
    prototype_context.set_schema_id(*schema.id());
    prototype_context.set_schema_variant_id(*schema_variant.id());
    prototype_context.set_component_id(*component.id());
    let prototype = WorkflowPrototype::new(
        ctx,
        *func.id(),
        serde_json::Value::Null,
        prototype_context,
        "prototype",
    )
    .await
    .expect("cannot create new prototype");

    // Apply the change set.
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not perform get by pk")
        .expect("could not get change set");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");
    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    ctx.update_visibility(Visibility::new_head(false));

    let (_, state, _, _) = WorkflowRunner::run(ctx, 0, *prototype.id(), *component.id())
        .await
        .expect("unable to run workflow");
    assert_eq!(
        state.error_message().expect("no error message found"),
        "oopsie!"
    );
    assert_eq!(state.status(), WorkflowRunnerStatus::Failure);
}

#[test]
async fn run(ctx: &mut DalContext) {
    let title = "Refresh Docker Image";
    let prototype = WorkflowPrototype::find_by_attr(ctx, "title", &title)
        .await
        .expect("unable to find workflow by attr")
        .pop()
        .expect("unable to find docker image resource refresh workflow prototype");

    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image")
        .await
        .expect("unable to find docker image schema")
        .pop()
        .expect("unable to find docker image");
    let (component, _) =
        Component::new_for_default_variant_from_schema(ctx, "systeminit/whiskers", *schema.id())
            .await
            .expect("cannot create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert!(component
        .resource(ctx)
        .await
        .expect("unable to fetch resource")
        .payload
        .is_none());

    // Apply the change set.
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not perform get by pk")
        .expect("could not get change set");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");
    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    ctx.update_visibility(Visibility::new_head(false));

    let (_runner, state, _func_bindings, _) =
        WorkflowRunner::run(ctx, 0, *prototype.id(), *component.id())
            .await
            .expect("unable to run workflow runner");
    assert_eq!(state.status(), WorkflowRunnerStatus::Success);

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let resource_attribute_value = Component::root_prop_child_attribute_value_for_component(
        ctx,
        *component.id(),
        RootPropChild::Resource,
    )
    .await
    .expect("unable to find attribute value");

    ctx.enqueue_job(DependentValuesUpdate::new(
        ctx.access_builder(),
        *ctx.visibility(),
        vec![*resource_attribute_value.id()],
    ))
    .await
    .expect("failed to enqueue job");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert!(component
        .resource(ctx)
        .await
        .expect("unable to fetch resource")
        .payload
        .is_some());
}
