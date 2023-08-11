use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, SiCliError};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use docker_api::Docker;

impl AppState {
    pub async fn check(&self, silent: bool) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "check-dependencies"}),
        );
        invoke(silent, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(silent: bool, is_preview: bool) -> CliResult<()> {
    if !silent {
        println!("Checking that the system is able to interact with the docker engine to control System Initiative...");
    }

    if is_preview {
        return Ok(());
    }

    let docker = Docker::unix("//var/run/docker.sock");
    if let Err(_e) = docker.ping().await {
        return Err(SiCliError::DockerEngine);
    }

    if !silent {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(100)
            .add_row(vec![
                Cell::new("Docker Engine Active").add_attribute(Attribute::Bold),
                Cell::new("    ✅    "),
            ]);

        println!("{table}");
    }

    Ok(())
}
