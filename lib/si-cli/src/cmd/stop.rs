use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};

impl AppState {
    pub async fn stop(&self) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "check-dependencies"}),
        );
        invoke(self, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, is_preview: bool) -> CliResult<()> {
    app.check(true).await?;

    if is_preview {
        println!("Stopped the following containers:");
    }

    for container_name in CONTAINER_NAMES.iter().rev() {
        let container_identifier = format!("local-{0}-1", container_name);
        if is_preview {
            println!("{}", container_identifier.clone());
            continue;
        }

        let existing = app
            .container_engine()
            .get_existing_container(container_identifier.clone())
            .await?;
        if existing.is_some() {
            println!("Stopping container {}", container_identifier.clone());
            app.container_engine()
                .stop_container(existing.unwrap().id.unwrap().to_string())
                .await?;
        }
    }

    if !is_preview {
        println!("All system components stopped...");
    }

    Ok(())
}
