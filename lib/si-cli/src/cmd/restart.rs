use crate::cmd::{start, stop};
use crate::CliResult;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "restart-system", "mode": mode}),
    );

    stop::invoke(posthog_client, mode.clone(), false).await?;
    start::invoke(posthog_client, mode.clone(), false).await?;

    Ok(())
}
