use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::CliResult;

impl AppState {
    pub async fn restart(&self) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "restart-system"}),
        );
        invoke(self).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState) -> CliResult<()> {
    app.stop().await?;
    app.start().await?;

    Ok(())
}
