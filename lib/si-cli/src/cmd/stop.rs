use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};
use docker_api::opts::{ContainerFilter, ContainerListOpts, ContainerStopOpts};
use docker_api::Docker;

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
        let docker = Docker::unix("//var/run/docker.sock");
        let filter = ContainerFilter::Name(container_identifier.clone());
        let list_opts = ContainerListOpts::builder()
            .filter([filter])
            .all(true)
            .build();
        let containers = docker
            .containers()
            .list(&list_opts)
            .await
            .expect("Issue making Docker Image Search");
        if !containers.is_empty() {
            let container = containers.first().unwrap();
            if let Some(state) = container.state.as_ref() {
                if *state == "running" {
                    let existing_id = container.id.as_ref().unwrap();
                    println!("Stopping Container: {}", container_identifier.clone());
                    docker
                        .containers()
                        .get(existing_id)
                        .stop(&ContainerStopOpts::builder().build())
                        .await
                        .expect("Issue stopping docker container");
                }
            }
        }
    }

    if !is_preview {
        println!("All system components stopped...");
    }

    Ok(())
}
