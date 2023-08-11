use crate::containers::{cleanup_image, delete_container, get_existing_container};
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::Docker;

impl AppState {
    pub async fn delete(&self) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "delete-system"}),
        );
        invoke(self, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, is_preview: bool) -> CliResult<()> {
    app.check(true).await?;

    let docker = Docker::unix("//var/run/docker.sock");

    if is_preview {
        println!("Deleted the following containers and associated images:");
    }

    for name in CONTAINER_NAMES.iter() {
        let container_name = format!("local-{0}-1", name);
        if is_preview {
            println!("{}", container_name);
            continue;
        }
        let container_summary = get_existing_container(&docker, container_name.clone()).await?;
        if let Some(container_summary) = container_summary {
            delete_container(&docker, container_summary, container_name.clone()).await?;
            cleanup_image(&docker, name.to_string()).await?;
        }
    }

    Ok(())
}
