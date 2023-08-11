use crate::key_management::{
    does_credentials_file_exist, get_credentials, get_si_data_dir, get_user_email,
    write_veritech_credentials,
};
use crate::state::AppState;
use crate::CliResult;
use inquire::{Password, PasswordDisplayMode};

impl AppState {
    pub async fn configure(&self, reconfigure: bool) -> CliResult<()> {
        self.track(
            get_user_email().await?,
            serde_json::json!({"command-name": "configure"}),
        );
        invoke(self.is_preview(), reconfigure).await?;
        Ok(())
    }
}

async fn invoke(_is_preview: bool, reconfigure: bool) -> CliResult<()> {
    let mut prompt_everything = false;
    let mut requires_rewrite = false;
    if !does_credentials_file_exist().await? || reconfigure {
        prompt_everything = true
    }

    // if the path doesn't exist, then we need to prompt for everything!
    let mut raw_creds = get_credentials().await?;
    let creds_path = get_si_data_dir().await?.join("si_credentials.toml");

    println!("System Initiative needs some credentials in order to be able to interact with AWS and Docker.");
    println!("The credentials are never sent back to System Initiative and can be inspected at the location:");
    println!("{}\n", creds_path.display());

    if prompt_everything || raw_creds.aws_access_key_id.is_empty() {
        let aws_access_key = Password::new("AWS Access Key ID")
            .with_display_toggle_enabled()
            .without_confirmation()
            .with_display_mode(PasswordDisplayMode::Full)
            .prompt();

        match aws_access_key {
            Ok(aws_access_key) => {
                raw_creds.aws_access_key_id = aws_access_key;
                requires_rewrite = true;
            }
            Err(_) => {
                println!("An error happened when asking for your AWS Access Key, try again later.")
            }
        }
    }

    if prompt_everything || raw_creds.aws_secret_access_key.is_empty() {
        let aws_secret_access_key = Password::new("AWS Secret Access Key")
            .with_display_toggle_enabled()
            .without_confirmation()
            .with_display_mode(PasswordDisplayMode::Masked)
            .prompt();

        match aws_secret_access_key {
            Ok(aws_secret_access_key) => {
                raw_creds.aws_secret_access_key = aws_secret_access_key;
                requires_rewrite = true;
            }
            Err(_) => println!(
                "An error happened when asking for your AWS Secret Access Key, try again later."
            ),
        }
    }

    if prompt_everything {
        let docker_hub_user_name = Password::new("Docker Hub User Name")
            .with_display_toggle_enabled()
            .without_confirmation()
            .with_display_mode(PasswordDisplayMode::Full)
            .prompt();

        match docker_hub_user_name {
            Ok(docker_hub_user_name) => {
                raw_creds.docker_hub_user_name = Some(docker_hub_user_name);
                requires_rewrite = true;
            }
            Err(_) => println!("Skipped adding a docker hub user name"),
        }
    }

    if prompt_everything {
        let docker_hub_token_or_password =
            Password::new("Docker Hub Password or Auth Token - either can be specified")
                .with_display_toggle_enabled()
                .without_confirmation()
                .with_display_mode(PasswordDisplayMode::Masked)
                .prompt();

        match docker_hub_token_or_password {
            Ok(docker_hub_token_or_password) => {
                raw_creds.docker_hub_credential = Some(docker_hub_token_or_password);
                requires_rewrite = true;
            }
            Err(_) => println!("Skipped adding a docker hub user name"),
        }
    }

    if prompt_everything
        || raw_creds.si_email.is_none()
        || raw_creds.si_email.clone().is_some_and(|e| e.is_empty())
    {
        let si_email = Password::new("Email used to log into System Initiative account")
            .with_display_toggle_enabled()
            .with_display_mode(PasswordDisplayMode::Full)
            .without_confirmation()
            .prompt();

        match si_email {
            Ok(si_email) => {
                raw_creds.si_email = Some(si_email);
                requires_rewrite = true;
            }
            Err(_) => println!("Skipped adding a SI email address"),
        }
    }

    if requires_rewrite {
        write_veritech_credentials(&raw_creds, creds_path).await?;
        println!("Credentials successfully stored!\n");
    }

    Ok(())
}
