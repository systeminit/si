use crate::args::{Commands, Engine};
use color_eyre::Result;
use si_cli::cmd::{check, delete, install, launch, report, restart, start, status, stop, update};
use telemetry_application::{prelude::*, TelemetryConfig};
use tokio::sync::oneshot::Sender;

mod args;

static VERSION: &str = include_str!("version.txt");

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("cli")
        .service_namespace("cli")
        .log_env_var_prefix("SI")
        .app_modules(vec!["si"])
        .build()?;
    let _telemetry = telemetry_application::init(config)?;
    let args = args::parse();
    let mode = args.mode();
    let is_preview = args.is_preview;

    debug!(arguments =?args, "parsed cli arguments");

    let (ph_client, ph_sender) = si_posthog::new().request_timeout_ms(3000).build()?;
    let (ph_done_sender, ph_done_receiver) = tokio::sync::oneshot::channel();

    tokio::spawn(wait_for_posthog_flush(ph_done_sender, ph_sender));

    println!(
        "{}\n\n",
        format_args!(
            "System Initiative Launcher is in {:?} mode",
            mode.to_string()
        )
    );

    // TODO: move this to be a CLI argument instead of env var
    #[allow(clippy::disallowed_methods)]
    let auth_api_host = std::env::var("AUTH_API").ok();

    if !matches!(args.command, Commands::Update(_)) {
        match update::find(VERSION, auth_api_host.as_deref()).await {
            Ok(update) => {
                if update.si.is_some() {
                    println!("Launcher update found, please run `si update` to install it");
                }

                if !update.containers.is_empty() {
                    println!("Containers update found, please run `si update` to install them");
                }
                println!();
            }
            Err(err) => {
                println!("Unable to retrieve updates: {err}");
            }
        }
    }

    if let Engine::Podman = args.engine() {
        println!("Podman isn't supported as an engine at this time! It's coming soon though...");
        return Ok(());
    }

    if is_preview {
        println!("Preview mode... System Initiative would have taken the following actions");
    }

    match args.command {
        Commands::Install(_args) => {
            install::invoke(&ph_client, mode.to_string(), is_preview).await?;
        }
        Commands::Check(_args) => {
            check::invoke(&ph_client, mode.to_string(), false, is_preview).await?;
        }
        Commands::Launch(_args) => {
            launch::invoke(&ph_client, mode.to_string())?;
        }
        Commands::Start(_args) => {
            start::invoke(&ph_client, mode.to_string(), is_preview).await?;
        }
        Commands::Delete(_args) => {
            delete::invoke(&ph_client, mode.to_string(), is_preview).await?;
        }
        Commands::Restart(_args) => {
            restart::invoke(&ph_client, mode.to_string())?;
        }
        Commands::Stop(_args) => {
            stop::invoke(&ph_client, mode.to_string(), is_preview).await?;
        }
        Commands::Update(args) => {
            update::invoke(
                VERSION,
                auth_api_host.as_deref(),
                &ph_client,
                mode.to_string(),
                args.skip_confirmation,
                args.binary,
            )
            .await?;
        }
        Commands::Status(args) => {
            status::invoke(&ph_client, mode.to_string(), args.show_logs, args.log_lines).await?;
        }
        Commands::Report(_args) => {
            report::invoke(&ph_client, mode.to_string())?;
        }
    }
    drop(ph_client);

    if let Err(e) = ph_done_receiver.await {
        println!("{}", e)
    }
    Ok(())
}

async fn wait_for_posthog_flush(done_sender: Sender<()>, sender: si_posthog::PosthogSender) {
    sender.run().await;
    done_sender
        .send(())
        .expect("Unable to push events to Posthog")
}
