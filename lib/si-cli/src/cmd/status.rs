use crate::containers::{get_container_logs, get_existing_container};
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use docker_api::Docker;

const RUNNING: &str = "    ✅    ";
const NOT_RUNNING: &str = "    ❌    ";

impl AppState {
    pub async fn status(&self, show_logs: bool, log_lines: usize) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "system-status"}),
        );
        invoke(show_logs, log_lines).await?;
        Ok(())
    }
}

async fn invoke(show_logs: bool, log_lines: usize) -> CliResult<()> {
    println!("Checking the status of System Initiative Software");
    let docker = Docker::unix("//var/run/docker.sock");
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Container Image").add_attribute(Attribute::Bold),
            Cell::new("State").add_attribute(Attribute::Bold),
        ]);

    let mut broken_containers = Vec::new();
    for name in CONTAINER_NAMES.iter() {
        let image_name = format!("systeminit/{0}:stable", name);
        let container_identifier = format!("local-{0}-1", name);
        let existing_container =
            get_existing_container(&docker, container_identifier.clone()).await?;
        let mut state = "".to_string();
        if let Some(existing) = existing_container {
            state = existing.state.unwrap();
            if state != "running" {
                broken_containers.push(image_name.clone());
            }
        } else {
            broken_containers.push(image_name.clone());
        }

        if show_logs {
            println!("\n\nShowing container logs for {0}", image_name.clone());
            get_container_logs(&docker, container_identifier.clone(), log_lines).await?;
        }

        table.add_row(vec![
            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
            Cell::new(if state == "running" {
                RUNNING
            } else {
                NOT_RUNNING
            }),
        ]);
    }

    println!("{table}");

    if broken_containers.is_empty() {
        println!("\nAll system components working as expected...")
    }

    Ok(())
}
