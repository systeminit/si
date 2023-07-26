use crate::cmd::check;
use crate::containers::has_existing_container;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::Docker;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "check-dependencies", "mode": mode}),
    );

    check::invoke(posthog_client, mode.clone(), true).await?;
    let docker = Docker::unix("//var/run/docker.sock");

    for name in CONTAINER_NAMES.iter() {
        let container_name = format!("dev-{0}-1", name);
        has_existing_container(&docker, container_name, true).await?;
    }

    Ok(())
}
