use crate::cmd::{start, stop};
use crate::key_management::get_user_email;
use crate::CliResult;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let email = get_user_email().await?;
    let _ = posthog_client.capture(
        "si-command",
        email,
        serde_json::json!({"name": "restart-system", "mode": mode}),
    );

    stop::invoke(posthog_client, mode.clone(), false).await?;
    start::invoke(posthog_client, mode.clone(), false).await?;

    Ok(())
}
