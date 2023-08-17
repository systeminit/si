use crate::args::{Commands, Engine};
use color_eyre::{eyre::eyre, Result};
use si_cli::{state::AppState, DockerClient};
use std::sync::Arc;
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

    let docker_sock = if let Some(sock) = args.docker_sock.clone() {
        sock
    } else {
        "".to_string()
    };

    let bind_host = args.bind_host.clone();
    let bind_port = args.bind_port;

    let current_version = VERSION.trim();

    debug!(arguments =?args, "parsed cli arguments");

    let (ph_client, ph_sender) = si_posthog::new().request_timeout_ms(3000).build()?;
    let (ph_done_sender, ph_done_receiver) = tokio::sync::oneshot::channel();

    tokio::spawn(wait_for_posthog_flush(ph_done_sender, ph_sender));

    let docker_socket_candidates = vec![
        #[allow(clippy::disallowed_methods)] // Used to determine a path relative to users's home
        std::path::Path::new(&std::env::var("HOME")?)
            .join(".docker")
            .join("run")
            .join("docker.sock"),
        std::path::Path::new("/var/run/docker.sock").to_path_buf(),
    ];

    let docker: DockerClient;
    if let "" = docker_sock.as_str() {
        let socket = docker_socket_candidates
            .iter()
            .find(|candidate| candidate.exists())
            .ok_or(eyre!(
            "failed to determine Docker socket location. Set a custom location using `--docker-sock` \
            or `SI_DOCKER_SOCK`; candidates={docker_socket_candidates:?}"
        ))?;
        docker = DockerClient::unix(socket)
    } else {
        println!("Checking for user supplied docker.sock");
        let path = std::path::Path::new(docker_sock.as_str()).to_path_buf();
        docker = DockerClient::unix(path);
    }

    let state = AppState::new(
        ph_client,
        Arc::from(current_version),
        Arc::from(mode.to_string()),
        is_preview,
        bind_host,
        bind_port,
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
        match state
            .find(&docker, current_version, auth_api_host.as_deref())
            .await
        {
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
            state.install(&docker).await?;
        }
        Commands::Check(_args) => {
            state.check(&docker, false).await?;
        }
        Commands::Launch(args) => {
            state.launch(args.metrics).await?;
        }
        Commands::Start(_args) => {
            state.start(&docker).await?;
        }
        Commands::Configure(args) => {
            state.configure(args.force_reconfigure).await?;
        }
        Commands::Delete(_args) => {
            state.delete(&docker).await?;
        }
        Commands::Restart(_args) => {
            state.restart(&docker).await?;
        }
        Commands::Stop(_args) => {
            state.stop(&docker).await?;
        }
        Commands::Update(args) => {
            state
                .update(
                    &docker,
                    current_version,
                    auth_api_host.as_deref(),
                    args.skip_confirmation,
                    args.binary,
                )
                .await?;
        }
        Commands::Status(args) => {
            state
                .status(&docker, args.show_logs, args.log_lines)
                .await?;
        } // Commands::Report(_args) => {
          //     state.report().await?;
          // }
    }

    drop(state);

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
