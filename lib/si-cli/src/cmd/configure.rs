use crate::key_management::{get_si_data_dir, write_veritech_credentials, Credentials};
use crate::CliResult;
use inquire::{Password, PasswordDisplayMode};
use si_posthog::PosthogClient;

pub async fn invoke(
    posthog_client: &PosthogClient,
    mode: String,
    _is_preview: bool,
    reconfigure: bool,
) -> CliResult<()> {
    let _ = posthog_client.capture(
        "si-command",
        "sally@systeminit.com",
        serde_json::json!({"name": "configure-system", "mode": mode}),
    );

    let si_data_dir = get_si_data_dir().await?;
    let credentials_path = si_data_dir.join("si_credentials.toml");

    if credentials_path.exists() && !reconfigure {
        return Ok(());
    }

    println!("System Initiative needs some credentials in order to be able to interact with AWS and Docker.");
    println!("The credentials are never sent back to System Initiative and can be inspected at the location:");
    println!("{}", credentials_path.display());

    let mut credentials = Credentials {
        aws_access_key_id: "".to_string(),
        aws_secret_access_key: "".to_string(),
        docker_hub_user_name: "".to_string(),
        docker_hub_credential: "".to_string(),
    };

    let aws_access_key = Password::new("AWS Access Key ID")
        .with_display_toggle_enabled()
        .without_confirmation()
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt();

    match aws_access_key {
        Ok(aws_access_key) => credentials.aws_access_key_id = aws_access_key,
        Err(_) => {
            println!("An error happened when asking for your AWS Access Key, try again later.")
        }
    }

    let aws_secret_access_key = Password::new("AWS Secret Access Key")
        .with_display_toggle_enabled()
        .without_confirmation()
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt();

    match aws_secret_access_key {
        Ok(aws_secret_access_key) => credentials.aws_secret_access_key = aws_secret_access_key,
        Err(_) => println!(
            "An error happened when asking for your AWS Secret Access Key, try again later."
        ),
    }

    let docker_hub_user_name = Password::new("Docker Hub User Name")
        .with_display_toggle_enabled()
        .without_confirmation()
        .with_display_mode(PasswordDisplayMode::Masked)
        .prompt();

    match docker_hub_user_name {
        Ok(docker_hub_user_name) => credentials.docker_hub_user_name = docker_hub_user_name,
        Err(_) => println!("Skipped adding a docker hub user name"),
    }

    let docker_hub_token_or_password =
        Password::new("Docker Hub Password or Auth Token - either can be specified")
            .with_display_toggle_enabled()
            .without_confirmation()
            .with_display_mode(PasswordDisplayMode::Masked)
            .prompt();

    match docker_hub_token_or_password {
        Ok(docker_hub_token_or_password) => {
            credentials.docker_hub_credential = docker_hub_token_or_password
        }
        Err(_) => println!("Skipped adding a docker hub user name"),
    }

    write_veritech_credentials(&credentials, credentials_path).await?;
    println!("Credentials successfully stored!");

    Ok(())
}
