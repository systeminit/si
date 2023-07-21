use crate::CliResult;
use inquire::{Confirm, Text};
use si_posthog::PosthogClient;

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "report-error", "mode": mode}),
    );
    let ans = Confirm::new("So, you'd like to report a bug?")
        .with_default(true)
        .with_help_message(
            "Please Note: We will collect some data from your system - OS, arch etc.",
        )
        .prompt();

    match ans {
        Ok(true) => println!(
            "We have collected your OS version, architecture and SI version from this installation",
        ),
        Ok(false) => println!("Whimp! ;)"),
        Err(_) => println!("Error: Try again later!"),
    }

    let info = Text::new("Do you want to provide us any other information?").prompt();

    match info {
        Ok(_) => println!("Thank you for making System Initiative better!!"),
        Err(_) => println!("Error: Try again later!"),
    }

    println!("Report received");

    Ok(())
}
