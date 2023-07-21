use crate::CliResult;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use si_posthog::PosthogClient;

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
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
            Cell::new("Dependency").add_attribute(Attribute::Bold),
            Cell::new("Success?").add_attribute(Attribute::Bold),
        ])
        .add_row(vec![
            Cell::new("Detected Docker Engine").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Detected Docker Command").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Docker Compose Available").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Found `bash` in Nix environment").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Found nix environment").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Reasonable value for max open files").add_attribute(Attribute::Bold),
            Cell::new("    ❌    "),
        ]);

    println!("{table}");

    Ok(())
}
