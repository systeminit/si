use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, SiCliError};

impl AppState {
    pub async fn launch(&self) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "launch-ui"}),
        );
        invoke().await?;
        Ok(())
    }
}

async fn invoke() -> CliResult<()> {
    let path = "http://localhost:8080";
    match open::that(path) {
        Ok(()) => Ok(()),
        Err(_err) => Err(SiCliError::FailToLaunch(path.to_string())),
    }
    .expect("issue opening url");

    Ok(())
}
