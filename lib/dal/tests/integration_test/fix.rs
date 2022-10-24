use dal::{
    workflow_runner::workflow_runner_state::WorkflowRunnerStatus, ActionPrototype,
    ConfirmationPrototype, ConfirmationResolver, DalContext, StandardModel, SystemId,
    WorkflowRunner,
};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    test,
};
use serde::Deserialize;

/// Expected output shape from running `skopeo inspect`, which is used by the builtin action
/// in this test module.
#[derive(Deserialize, Debug)]
struct SkopeoOutput {
    #[serde(rename = "Name")]
    name: String,
}

#[test]
async fn confirmation_to_action(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let payload = harness
        .create_component(ctx, "systeminit/whiskers", Builtin::DockerImage)
        .await;

    let confirmation_prototype = ConfirmationPrototype::get_by_component_and_name(
        ctx,
        payload.component_id,
        "Has docker image resource?",
        payload.schema_id,
        payload.schema_variant_id,
        SystemId::NONE,
    )
    .await
    .expect("could not find confirmation prototype")
    .expect("no confirmation prototype found");

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

    let run_id = rand::random();
    let (_runner, runner_state, func_binding_return_values, _created_resources, _updated_resources) =
        WorkflowRunner::run(
            ctx,
            run_id,
            filtered_action_prototype.workflow_prototype_id(),
            payload.component_id,
        )
        .await
        .expect("could not perform workflow runner run");
    assert_eq!(runner_state.status(), WorkflowRunnerStatus::Success);

    let mut maybe_skopeo_output_name: Option<String> = None;
    for func_binding_return_value in func_binding_return_values {
        for stream in func_binding_return_value
            .get_output_stream(ctx)
            .await
            .expect("could not get output stream from func binding return value")
            .unwrap_or_default()
        {
            let maybe_skopeo_output: serde_json::Result<SkopeoOutput> =
                serde_json::from_str(&stream.message);
            if let Ok(skopeo_output) = maybe_skopeo_output {
                if maybe_skopeo_output_name.is_some() {
                    panic!(
                        "already found skopeo output with name: {:?}",
                        maybe_skopeo_output_name
                    );
                }
                maybe_skopeo_output_name = Some(skopeo_output.name);
            }
        }
    }
    let skopeo_outputname =
        maybe_skopeo_output_name.expect("could not find name via skopeo output");
    assert_eq!(skopeo_outputname, "docker.io/systeminit/whiskers");
}
