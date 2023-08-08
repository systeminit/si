use crate::containers::{download_missing_containers, missing_containers};
use crate::key_management::get_user_email;
use crate::CliResult;
use si_posthog::PosthogClient;

pub async fn invoke(
    posthog_client: &PosthogClient,
    mode: String,
    is_preview: bool,
) -> CliResult<()> {
    let email = get_user_email().await?;
    let _ = posthog_client.capture(
        "si-command",
        email,
        serde_json::json!({"name": "install", "mode": mode}),
    );

    let missing_containers = missing_containers().await?;
    if missing_containers.is_empty() {
        println!("All containers downloaded\n");
        return Ok(());
    }

    if is_preview {
        println!("Downloaded the following containers:");
        for missing_container in missing_containers.clone() {
            println!("{}", missing_container);
        }
        return Ok(());
    }

    println!("Downloading the containers required to run System Initiative");
    download_missing_containers(missing_containers).await?;

    Ok(())
}
