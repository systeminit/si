use crate::containers::{get_container_logs, has_existing_container};
use crate::{CliResult, CONTAINER_NAMES};
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use docker_api::Docker;
use si_posthog::PosthogClient;

const RUNNING: &str = "    ✅    ";
const NOT_RUNNING: &str = "    ❌    ";

pub async fn invoke(
    posthog_client: &PosthogClient,
    mode: String,
    show_logs: bool,
    log_lines: usize,
) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "check-dependencies", "mode": mode}),
    );
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
        let is_running =
            has_existing_container(&docker, container_identifier.clone(), false).await?;
        if !is_running {
            broken_containers.push(image_name.clone());
        }
        if show_logs {
            println!("\n\nShowing container logs for {0}", image_name.clone());
            get_container_logs(&docker, container_identifier.clone(), log_lines).await?;
        }

        table.add_row(vec![
            Cell::new(image_name.clone()).add_attribute(Attribute::Bold),
            Cell::new(if is_running { RUNNING } else { NOT_RUNNING }),
        ]);
    }

    println!("{table}");

    if broken_containers.is_empty() {
        println!("\nAll system components working as expected...")
    }

    Ok(())
}
