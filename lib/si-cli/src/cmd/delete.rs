use crate::containers::DockerClient;
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};

impl AppState {
    pub async fn delete(
        &self,
        docker: &DockerClient,
        keep_images: bool
    ) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "delete-system"}),
        );
        invoke(self, docker, self.is_preview(), keep_images).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, docker: &DockerClient, is_preview: bool, keep_images: bool) -> CliResult<()> {
    app.check(docker, true).await?;

    if is_preview {
        println!("Deleted the following containers and associated images:");
    }

    for name in CONTAINER_NAMES.iter() {
        let container_name = format!("local-{0}-1", name);
        if is_preview {
            println!("{}", container_name);
            continue;
        }
        let container_summary = docker
            .get_existing_container(container_name.clone())
            .await?;
        if let Some(container_summary) = container_summary {
            docker
                .delete_container(container_summary, container_name.clone())
                .await?;

            if !keep_images {
                docker.cleanup_image(name.to_string()).await?;
            }
        }
    }

    Ok(())
}
