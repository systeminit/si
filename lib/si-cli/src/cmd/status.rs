use comfy_table::presets::UTF8_FULL;
use comfy_table::*;

use crate::containers::DockerClient;
use crate::key_management::get_user_email;
use crate::state::AppState;
use crate::{CliResult, CONTAINER_NAMES};

const RUNNING: &str = "    ✅    ";
const NOT_RUNNING: &str = "    ❌    ";
const WAITING: &str = "    🕒    ";

impl AppState {
    pub async fn status(
        &self,
        docker: &DockerClient,
        show_logs: bool,
        log_lines: usize,
    ) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "system-status"}),
        );
        invoke(self, docker, show_logs, log_lines).await?;
        Ok(())
    }
}

#[derive(Debug)]
struct Status {
    name: String,
    state: ContainerState,
    version: String,
}

#[derive(Debug, PartialEq)]
enum ContainerState {
    Running,
    NotRunning,
    Waiting,
}

async fn invoke(app: &AppState, docker: &DockerClient, show_logs: bool, log_lines: usize) -> CliResult<()> {
    println!("Checking the status of System Initiative Software");

    let mut container_status = Vec::new();

    let mut all_running = true;
    for name in CONTAINER_NAMES.iter() {
        let image_name = format!("systeminit/{0}:stable", name);
        let container_identifier = format!("local-{0}-1", name);
        let existing_container = docker
            .get_existing_container(container_identifier.clone())
            .await?;
        let mut version = "".to_string();
        let mut state = ContainerState::NotRunning;
        if let Some(container) = existing_container {
            version = container
                .labels
                .unwrap()
                .get("org.opencontainers.image.version")
                .unwrap()
                .to_string();
            let raw_state = container.state.unwrap();
            if raw_state == "running" {
                state = ContainerState::Running;
            } else {
                all_running = false;
            }
        }

        if show_logs {
            println!("\n\nShowing container logs for {0}", image_name.clone());
            docker
                .get_container_logs(container_identifier.clone(), log_lines)
                .await?;
        }

        if container_identifier == "local-web-1" {
            let web_path = format!("http://{0}:{1}/", app.bind_host(), app.bind_port());
            let resp = reqwest::get(web_path).await;
            if resp.is_err() && state == ContainerState::Running {
                state = ContainerState::Waiting;
            }
        }

        if container_identifier == "local-sdf-1" {
            let sdf_path = "http://localhost:5156/api/";
            let resp = reqwest::get(sdf_path).await;
            if resp.is_err() && state == ContainerState::Running {
                state = ContainerState::Waiting;
            }
        }

        container_status.push(Status {
            name: image_name,
            state,
            version,
        })
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Container Image").add_attribute(Attribute::Bold),
            Cell::new("State").add_attribute(Attribute::Bold),
            Cell::new("Container Version").add_attribute(Attribute::Bold),
        ]);
    for container_status in container_status {
        table.add_row(vec![
            Cell::new(container_status.name).add_attribute(Attribute::Bold),
            Cell::new(match container_status.state {
                ContainerState::Running => RUNNING,
                ContainerState::NotRunning => NOT_RUNNING,
                ContainerState::Waiting => WAITING,
            }),
            Cell::new(container_status.version),
        ]);
    }
    println!("{table}");

    if all_running {
        println!("\nAll system components working as expected...")
    }

    Ok(())
}
