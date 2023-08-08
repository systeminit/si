use crate::key_management::get_user_email;
use crate::{CliResult, SiCliError};
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let email = get_user_email().await?;
    let _ = posthog_client.capture(
        "si-command",
        email,
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
