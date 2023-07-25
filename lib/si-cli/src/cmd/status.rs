use crate::containers::{get_non_running_containers, running_systeminit_containers_list};
use crate::CliResult;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use si_posthog::PosthogClient;

pub async fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "check-dependencies", "mode": mode}),
    );
    println!("Preparing for System Initiative Installation");
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Container Image").add_attribute(Attribute::Bold),
            Cell::new("Names").add_attribute(Attribute::Bold),
            Cell::new("State").add_attribute(Attribute::Bold),
        ]);

    let containers = running_systeminit_containers_list().await?;
    for container in containers {
        table.add_row(vec![
            Cell::new(container.image.unwrap_or("-".to_string()).to_string())
                .add_attribute(Attribute::Bold),
            Cell::new(
                container
                    .names
                    .unwrap_or(Vec::new())
                    .get(0)
                    .unwrap_or(&"-".to_string())
                    .to_string(),
            ),
            Cell::new(container.state.unwrap_or("-".to_string()).to_string()),
        ]);
    }

    println!("{table}");

    let non_started_containers = get_non_running_containers().await?;
    println!(
        "\n System is running?: {0}",
        non_started_containers.is_empty()
    );

    Ok(())
}
