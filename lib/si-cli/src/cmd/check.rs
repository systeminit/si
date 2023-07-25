use crate::dependencies::check_system_dependencies;
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

    println!("Checking that the system satisfies all the dependencies needed to run System Initiative...");
    let check_installation = check_system_dependencies().await?;
    if !check_installation {
        println!("System is not ready to install SI");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Dependency").add_attribute(Attribute::Bold),
            Cell::new("Success?").add_attribute(Attribute::Bold),
        ])
        .add_row(vec![
            Cell::new("Detected Docker Engine").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ]);

    println!("{table}");

    Ok(())
}
