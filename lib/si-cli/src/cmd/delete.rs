use crate::cmd::check;
use crate::containers::{cleanup_image, has_existing_container};
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::Docker;
use si_posthog::PosthogClient;

pub async fn invoke(
    posthog_client: &PosthogClient,
    mode: String,
    is_preview: bool,
) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "check-dependencies", "mode": mode}),
    );

    check::invoke(posthog_client, mode.clone(), true, is_preview).await?;
    let docker = Docker::unix("//var/run/docker.sock");

    if is_preview {
        println!("Deleted the following containers and associated images:");
    }

    for name in CONTAINER_NAMES.iter() {
        let container_name = format!("dev-{0}-1", name);
        if is_preview {
            println!("{}", container_name);
            continue;
        }
        has_existing_container(&docker, container_name, true).await?;
        cleanup_image(&docker, name.to_string()).await?;
    }

    Ok(())
}
