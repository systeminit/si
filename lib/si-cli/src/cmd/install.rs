use crate::containers::DockerClient;
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::CliResult;

impl AppState {
    pub async fn install(&self, docker: &DockerClient) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "install"}),
        );
        invoke(docker, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(docker: &DockerClient, is_preview: bool) -> CliResult<()> {
    let missing_containers = docker.missing_containers().await?;
    if missing_containers.is_empty() {
        println!("All containers downloaded\n");
        return Ok(());
    }

    if is_preview {
        println!("Downloaded the following containers:");
        for missing_container in missing_containers.clone() {
            println!("{}", missing_container);
        }
        return Ok(());
    }

    println!("Downloading the containers required to run System Initiative");
    docker
        .download_missing_containers(missing_containers)
        .await?;

    Ok(())
}
