use crate::{CliResult, SiCliError};
use si_posthog::PosthogClient;

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "launch-ui", "mode": mode}),
    );
    let path = "http://localhost:8080";
    match open::that(path) {
        Ok(()) => Ok(()),
        Err(_err) => Err(SiCliError::FailToLaunch(path.to_string())),
    }
    .expect("issue opening url");

    Ok(())
}
