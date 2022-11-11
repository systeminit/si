use dal::action_prototype::ActionKind;
use dal::{
    workflow_runner::workflow_runner_state::WorkflowRunnerStatus, ActionPrototype,
    ActionPrototypeContext, ConfirmationPrototype, ConfirmationPrototypeContext,
    ConfirmationResolver, ConfirmationResolverId, DalContext, Fix, FixBatch, FixCompletionStatus,
    Func, StandardModel, SystemId, WorkflowPrototype, WorkflowPrototypeContext,
    WorkflowPrototypeId, WorkflowRunner,
};
use dal_test::helpers::component_payload::ComponentPayload;
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};

#[test]
async fn confirmation_to_action(ctx: &DalContext) {
    let (payload, _confirmation_resolver_id, action_workflow_prototype_id, _action_name) =
        setup_confirmation_resolver_and_get_action_prototype(ctx).await;

    let run_id = rand::random();
    let (_runner, runner_state, func_binding_return_values, _created_resources, _updated_resources) =
        WorkflowRunner::run(
            ctx,
            run_id,
            action_workflow_prototype_id,
            payload.component_id,
            true,
        )
        .await
        .expect("could not perform workflow runner run");
    assert_eq!(runner_state.status(), WorkflowRunnerStatus::Success);

    let mut maybe_skopeo_output_name = None;
    for func_binding_return_value in &func_binding_return_values {
        maybe_skopeo_output_name = maybe_skopeo_output_name.or_else(|| {
            func_binding_return_value
                .value()
                .and_then(|v| v.pointer("/value/Name"))
                .and_then(|v| v.as_str())
        });
    }
    assert_eq!(
        maybe_skopeo_output_name,
        Some("docker.io/systeminit/whiskers")
    );
}

#[test]
async fn confirmation_to_fix(ctx: &DalContext) {
    let (payload, confirmation_resolver_id, action_workflow_prototype_id, action_name) =
        setup_confirmation_resolver_and_get_action_prototype(ctx).await;

    // Create the batch.
    let mut batch = FixBatch::new(ctx, "toddhoward@systeminit.com")
        .await
        .expect("could not create fix execution batch");
    assert!(batch.started_at().is_none());
    assert!(batch.finished_at().is_none());
    assert!(batch.completion_status().is_none());

    // Create all fix(es) before starting the batch.
    let mut fix = Fix::new(
        ctx,
        *batch.id(),
        confirmation_resolver_id,
        payload.component_id,
    )
    .await
    .expect("could not create fix");
    assert!(fix.started_at().is_none());
    assert!(fix.finished_at().is_none());
    assert!(fix.completion_status().is_none());

    // NOTE(nick): batches are stamped as started inside their job.
    batch
        .stamp_started(ctx)
        .await
        .expect("could not stamp batch as started");
    assert!(batch.started_at().is_some());
    assert!(batch.finished_at().is_none());
    assert!(batch.completion_status().is_none());

    let run_id = rand::random();
    fix.run(ctx, run_id, action_workflow_prototype_id, action_name, true)
        .await
        .expect("could not run fix");
    assert!(fix.started_at().is_some());
    assert!(fix.finished_at().is_some());
    let completion_status = fix
        .completion_status()
        .expect("no completion status found for fix");
    assert_eq!(completion_status, &FixCompletionStatus::Success);

    // NOTE(nick): batches are stamped as finished inside their job.
    let batch_completion_status = batch
        .stamp_finished(ctx)
        .await
        .expect("could not complete batch");
    assert!(batch.finished_at().is_some());
    assert_eq!(
        batch
            .completion_status()
            .expect("no completion status for batch"),
        &FixCompletionStatus::Success
    );
    assert_eq!(batch_completion_status, FixCompletionStatus::Success);

    let found_batch = fix
        .fix_batch(ctx)
        .await
        .expect("could not get fix execution batch")
        .expect("no fix execution batch found");
    assert_eq!(batch.id(), found_batch.id());
}

async fn setup_confirmation_resolver_and_get_action_prototype(
    ctx: &DalContext,
) -> (
    ComponentPayload,
    ConfirmationResolverId,
    WorkflowPrototypeId,
    String,
) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let payload = harness
        .create_component(ctx, "systeminit/whiskers", Builtin::DockerImage)
        .await;

    let func_name = "si:resourceExistsConfirmation";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("unable to find func")
        .pop()
        .expect("func not found");
    let context = ConfirmationPrototypeContext {
        schema_id: payload.schema_id,
        schema_variant_id: payload.schema_variant_id,
        ..Default::default()
    };
    let confirmation_prototype =
        ConfirmationPrototype::new(ctx, "Create Docker Image", *func.id(), context)
            .await
            .expect("unable to create confirmation prototype");

    let func_name = "si:dockerImageRefreshWorkflow";
    let func = Func::find_by_attr(ctx, "name", &func_name)
        .await
        .expect("unable to find func")
        .pop()
        .expect("unable to find func");

    let title = "Docker Image Refresh - Test";
    let context = WorkflowPrototypeContext {
        schema_id: payload.schema_id,
        schema_variant_id: payload.schema_variant_id,
        ..Default::default()
    };
    let workflow_prototype =
        WorkflowPrototype::new(ctx, *func.id(), serde_json::Value::Null, context, title)
            .await
            .expect("unable to create workflow prototype");

    let name = "create";
    let context = ActionPrototypeContext {
        schema_id: payload.schema_id,
        schema_variant_id: payload.schema_variant_id,
        ..Default::default()
    };
    ActionPrototype::new(
        ctx,
        *workflow_prototype.id(),
        name,
        ActionKind::Other,
        context,
    )
    .await
    .expect("unable to create action prototype");

    let confirmation_resolver = confirmation_prototype
        .run(ctx, payload.component_id, SystemId::NONE)
        .await
        .expect("could not run confirmation prototype");

    let mut found_confirmation_resolvers = ConfirmationResolver::list(ctx)
        .await
        .expect("could not list confirmation resolvers");
    let found_confirmation_resolver = found_confirmation_resolvers
        .pop()
        .expect("found confirmation resolvers is empty");
    assert!(found_confirmation_resolvers.is_empty());
    assert_eq!(found_confirmation_resolver.id(), confirmation_resolver.id());

    let expected_action_name = "create";
    let mut filtered_action_prototypes = confirmation_resolver
        .recommended_actions(ctx)
        .await
        .expect("could not find recommended actions from confirmation resolver")
        .into_iter()
        .filter(|a| a.name() == expected_action_name)
        .collect::<Vec<ActionPrototype>>();
    let filtered_action_prototype = filtered_action_prototypes
        .pop()
        .expect("empty filtered action prototypes");
    assert!(filtered_action_prototypes.is_empty());
    assert_eq!(filtered_action_prototype.name(), expected_action_name);

    let found_action_prototype = ActionPrototype::find_by_name(
        ctx,
        expected_action_name,
        payload.schema_id,
        payload.schema_variant_id,
        SystemId::NONE,
    )
    .await
    .expect("could not find action prototype")
    .expect("no action prototype found");

    assert_eq!(found_action_prototype.id(), filtered_action_prototype.id());

    (
        payload,
        *found_confirmation_resolver.id(),
        found_action_prototype.workflow_prototype_id(),
        found_action_prototype.name().to_string(),
    )
}
