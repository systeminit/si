use crate::cmd::check;
use crate::containers::delete_existing_container_id;
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
        delete_existing_container_id(&docker, container_name).await?;
    }

    Ok(())
}
