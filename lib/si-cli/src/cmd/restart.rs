use crate::containers::DockerClient;
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::CliResult;

impl AppState {
    pub async fn restart(&self, docker: &DockerClient) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "restart-system"}),
        );
        invoke(self, docker).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, docker: &DockerClient) -> CliResult<()> {
    app.stop(docker).await?;
    app.start(docker).await?;

    Ok(())
}
