use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, SiCliError};

impl AppState {
    pub async fn launch(&self, launch_metrics: bool) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "launch-ui"}),
        );
        invoke(launch_metrics).await?;
        Ok(())
    }
}

async fn invoke(launch_metrics: bool) -> CliResult<()> {
    let path = if launch_metrics {
        "http://localhost:16686"
    } else {
        "http://localhost:8080"
    };

    println!("Opening URL: {}", path);
    match open::that(path) {
        Ok(()) => Ok(()),
        Err(_err) => Err(SiCliError::FailToLaunch(path.to_string())),
    }
    .expect("issue opening url");

    Ok(())
}
