use crate::containers::{download_missing_containers, missing_containers};
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::CliResult;

impl AppState {
    pub async fn install(&self) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "install"}),
        );
        invoke(self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(is_preview: bool) -> CliResult<()> {
    let missing_containers = missing_containers().await?;
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
    download_missing_containers(missing_containers).await?;

    Ok(())
}
