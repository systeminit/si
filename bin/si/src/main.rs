use crate::args::{Commands, Engine};
use color_eyre::Result;
use si_cli::engine::docker_engine::DockerEngine;
use si_cli::engine::podman_engine::PodmanEngine;
use si_cli::state::AppState;
use std::sync::Arc;
use telemetry_application::prelude::*;
use tokio::sync::oneshot::Sender;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

mod args;

static VERSION: &str = include_str!("version.txt");

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<()> {
    let shutdown_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("cli")
        .service_namespace("cli")
        .log_env_var_prefix("SI")
        .app_modules(vec!["si"])
        .signal_handlers(false)
        .build()?;
    let _telemetry = telemetry_application::init(config, &task_tracker, shutdown_token.clone())?;
    let args = args::parse();
    let mode = args.mode();
    let is_preview = args.is_preview;
    let mut engine = args.engine();

    if let Some(podman_sock) = args.podman_sock.clone() {
        if podman_sock.contains("podman.sock") && engine != Engine::Podman {
            println!("WARNING: A `podman_sock` location parameter was passed but no Podman engine was \
            chosen. Continuing with the assumption you meant to choose Podman as the container engine...\n");
            engine = Engine::Podman
        }
    }

    if let Some(docker_sock) = args.docker_sock.clone() {
        if docker_sock.contains("docker.sock") && engine != Engine::Docker {
            println!("WARNING: A `docker_sock` location parameter was passed but no Docker engine was \
            chosen. Continuing with the assumption you meant to choose Docker as the container engine...\n");
            engine = Engine::Docker
        }
    }

    let engine = match engine {
        Engine::Docker => DockerEngine::new(args.docker_sock.clone()).await?,
        Engine::Podman => PodmanEngine::new(args.podman_sock.clone()).await?,
    };

    let web_host = args.web_host.clone();
    let web_port = args.web_port;

    let sdf_host = args.sdf_host.clone();
    let sdf_port = args.sdf_port;

    let current_version = VERSION.trim();

    debug!(arguments =?args, "parsed cli arguments");

    let (ph_client, ph_sender) = si_posthog::new().request_timeout_ms(3000).build()?;
    let (ph_done_sender, ph_done_receiver) = tokio::sync::oneshot::channel();

    tokio::spawn(wait_for_posthog_flush(ph_done_sender, ph_sender));

    let state = AppState::new(
        ph_client,
        Arc::from(current_version),
        Arc::from(mode.to_string()),
        is_preview,
        web_host,
        web_port,
        sdf_host,
        sdf_port,
        args.with_function_debug_logs,
        Arc::from(engine),
    );

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
        match state.find(current_version, auth_api_host.as_deref()).await {
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

    if is_preview {
        println!("Preview mode... System Initiative would have taken the following actions");
    }

    task_tracker.close();

    match args.command {
        Commands::Install(_args) => {
            state.install().await?;
        }
        Commands::Check(_args) => {
            state.check(false).await?;
        }
        Commands::Launch(args) => {
            state.launch(args.metrics).await?;
        }
        Commands::Start(_args) => {
            state.start().await?;
        }
        Commands::Configure(args) => {
            state.configure(args.force_reconfigure).await?;
        }
        Commands::Delete(args) => {
            state.delete(args.keep_images).await?;
        }
        Commands::Restart(_args) => {
            state.restart().await?;
        }
        Commands::Stop(_args) => {
            state.stop().await?;
        }
        Commands::Update(args) => {
            state
                .update(
                    current_version,
                    auth_api_host.as_deref(),
                    args.skip_confirmation,
                    args.binary,
                )
                .await?;
        }
        Commands::Status(args) => {
            state.status(args.show_logs, args.log_lines).await?;
        } // Commands::Report(_args) => {
          //     state.report().await?;
          // }
    }

    drop(state);

    // TODO(fnichol): this will eventually go into the signal handler code but at the moment in
    // si's case, this doesn't really exist anywhere. At this moment in the program however, the
    // command to run has shut down so it's an appropriate time to cancel other remaining tasks and
    // wait on their graceful shutdowns
    {
        shutdown_token.cancel();
        task_tracker.wait().await;
    }

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
