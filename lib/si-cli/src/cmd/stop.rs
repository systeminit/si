use crate::cmd::check;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::opts::{ContainerFilter, ContainerListOpts, ContainerStopOpts};
use docker_api::Docker;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "stop-system", "mode": mode}),
    );

    check::invoke(posthog_client, mode.clone(), true).await?;

    for container_name in CONTAINER_NAMES.iter() {
        let container_identifier = format!("dev-{0}-1", container_name);
        let docker = Docker::unix("//var/run/docker.sock");
        let filter = ContainerFilter::Name(container_identifier.clone());
        let list_opts = ContainerListOpts::builder()
            .filter([filter])
            .all(true)
            .build();
        let containers = docker
            .containers()
            .list(&list_opts)
            .await
            .expect("Issue making Docker Image Search");
        if !containers.is_empty() {
            let container = containers.first().unwrap();
            if let Some(state) = container.state.as_ref() {
                if *state == "running" {
                    let existing_id = container.id.as_ref().unwrap();
                    println!("Stopping Container: {}", container_identifier.clone());
                    docker
                        .containers()
                        .get(existing_id)
                        .stop(&ContainerStopOpts::builder().build())
                        .await
                        .expect("Issue stopping docker container");
                }
            }
        }
    }

    println!("All system components stopped...");

    Ok(())
}
