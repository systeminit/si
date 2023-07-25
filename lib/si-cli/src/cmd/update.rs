use crate::CliResult;
use inquire::Confirm;
use si_posthog::PosthogClient;

pub fn invoke(posthog_client: &PosthogClient, mode: String) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "update-launcher", "mode": mode}),
    );
    let ans = Confirm::new("Are you sure you want to update this launcher?")
        .with_default(false)
        .with_help_message("Please Note: No container data is backed up during update!")
        .prompt();

    match ans {
        Ok(true) => println!("That's awesome! Let's do this"),
        Ok(false) => println!("Whimp! ;)"),
        Err(_) => println!("Error: Try again later!"),
    }

    Ok(())
}
