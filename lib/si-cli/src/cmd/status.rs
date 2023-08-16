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
        invoke(docker, show_logs, log_lines).await?;
        Ok(())
    }
}

async fn invoke(docker: &DockerClient, show_logs: bool, log_lines: usize) -> CliResult<()> {
    println!("Checking the status of System Initiative Software");
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Container Image").add_attribute(Attribute::Bold),
            Cell::new("State").add_attribute(Attribute::Bold),
        ]);

    let mut all_running = true;
    for name in CONTAINER_NAMES.iter() {
        let image_name = format!("systeminit/{0}:stable", name);
        let container_identifier = format!("local-{0}-1", name);
        let existing_container = docker
            .get_existing_container(container_identifier.clone())
            .await?;
        let mut state = "".to_string();
        if let Some(existing) = existing_container {
            state = existing.state.unwrap();
            if state != "running" {
                all_running = false;
            }
        } else {
            all_running = false;
        }

        if show_logs {
            println!("\n\nShowing container logs for {0}", image_name.clone());
            docker
                .get_container_logs(container_identifier.clone(), log_lines)
                .await?;
        }

        if container_identifier == "local-web-1" {
            let web_path = "http://localhost:8080/";
            let resp = reqwest::get(web_path).await;
            match resp {
                Ok(x) => {
                    if x.status().as_u16() == 200 && state == "running" {
                        table.add_row(vec![
                            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                            Cell::new(RUNNING),
                        ]);
                    } else {
                    }
                }
                Err(_) => {
                    if state == "running" {
                        table.add_row(vec![
                            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                            Cell::new(WAITING),
                        ]);
                        all_running = false;
                    } else {
                        table.add_row(vec![
                            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                            Cell::new(NOT_RUNNING),
                        ]);
                    }
                }
            }
        } else if container_identifier == "local-sdf-1" {
            let sdf_path = "http://localhost:5156/api/";
            let resp = reqwest::get(sdf_path).await;
            match resp {
                Ok(x) => {
                    if x.status().as_u16() == 200 && state == "running" {
                        table.add_row(vec![
                            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                            Cell::new(RUNNING),
                        ]);
                    }
                }
                Err(_) => {
                    if state == "running" {
                        table.add_row(vec![
                            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                            Cell::new(WAITING),
                        ]);
                        all_running = false;
                    } else {
                        table.add_row(vec![
                            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                            Cell::new(NOT_RUNNING),
                        ]);
                    }
                }
            }
        } else {
            table.add_row(vec![
                Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
                Cell::new(if state == "running" {
                    RUNNING
                } else {
                    NOT_RUNNING
                }),
            ]);
        }
    }

    println!("{table}");

    if all_running {
        println!("\nAll system components working as expected...")
    }

    Ok(())
}
