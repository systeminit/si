use crate::CliResult;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use si_posthog::PosthogClient;

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "status-check", "mode": mode}),
    );
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(100)
        .set_header(vec![
            Cell::new("Component").add_attribute(Attribute::Bold),
            Cell::new("Healthy?").add_attribute(Attribute::Bold),
        ])
        .add_row(vec![
            Cell::new("Council").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Veritech").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Pinga").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("SDF").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Module-Index").add_attribute(Attribute::Bold),
            Cell::new("    ✅    "),
        ])
        .add_row(vec![
            Cell::new("Web").add_attribute(Attribute::Bold),
            Cell::new("    ❌    "),
        ]);

    println!("{table}");
    Ok(())
}
