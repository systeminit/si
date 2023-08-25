use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, SiCliError};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

impl AppState {
    pub async fn check(&self, silent: bool) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "check-dependencies"}),
        );
        invoke(self, silent, self.is_preview()).await?;
        Ok(())
    }
}

async fn invoke(app: &AppState, silent: bool, is_preview: bool) -> CliResult<()> {
    if !silent {
        println!("Checking that the system is able to interact with the container engine to control System Initiative...");
    }

    if is_preview {
        return Ok(());
    }

    if let Err(_e) = app.container_engine().ping().await {
        return Err(SiCliError::ContainerEngine);
    }

    if !silent {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(100)
            .add_row(vec![
                Cell::new("Container Engine Active").add_attribute(Attribute::Bold),
                Cell::new("    ✅    "),
            ]);

        println!("{table}");
    }

    Ok(())
}
