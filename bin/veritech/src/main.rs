use std::time::Duration;

use args::NAME;
use innit_client::InnitClient;
use si_service::prelude::*;
use veritech_server::Server;

use crate::args::load_config_with_provider;

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

const DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 20);

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = args::parse();

    match args.tokio_cpu_cores() {
        Some(tokio_cpu_cores) => {
            rt::block_on_with_core_affinity(BIN_NAME, async_main(args), tokio_cpu_cores)
        }
        None => rt::block_on(BIN_NAME, async_main(args)),
    }
}

async fn async_main(args: args::Args) -> Result<()> {
    let main_tracker = TaskTracker::new();
    let main_token = CancellationToken::new();
    let endpoints_tracker = TaskTracker::new();
    let endpoints_token = CancellationToken::new();
    let telemetry_tracker = TaskTracker::new();
    let telemetry_token = CancellationToken::new();

    let (mut telemetry, telemetry_shutdown) = {
        let config = TelemetryConfig::builder()
            .force_color(args.force_color.then_some(true))
            .no_color(args.no_color.then_some(true))
            .log_format(if args.log_json {
                LogFormat::Json
            } else {
                Default::default()
            })
            .log_file_directory(args.log_file_directory.clone())
            .tokio_console(args.tokio_console)
            .service_name(BIN_NAME)
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec![BIN_NAME, LIB_NAME])
            .interesting_modules(vec!["naxum", "si_data_nats", "si_service"])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };

    startup::startup(BIN_NAME).await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    let graceful_shutdown_timeout = match args.graceful_shutdown_timeout_secs {
        Some(provided) => Duration::from_secs(provided),
        None => DEFAULT_GRACEFUL_SHUTDOWN_TIMEOUT,
    };
    debug!(arguments =?args, "parsed cli arguments");

    debug!("creating innit-client...");
    let provider = Some(InnitClient::new_from_environment(NAME.to_string()).await?);
    let config = load_config_with_provider(args, provider).await?;
    debug!(?config, "computed configuration");

    let endpoints_server = if config.service_endpoints().enabled {
        let endpoints = veritech_server::DefaultServiceEndpoints::from_config("veritech", &config)?;
        Some(veritech_server::EndpointsServer::new(
            std::sync::Arc::new(endpoints),
            config.service_endpoints().clone(),
            endpoints_token.clone(),
        ))
    } else {
        None
    };

    let (server, maybe_heartbeat_app) = Server::from_config(config, main_token.clone()).await?;

    if let Some(mut heartbeat_app) = maybe_heartbeat_app {
        main_tracker.spawn(async move { heartbeat_app.run().await });
    }
    main_tracker.spawn(async move {
        info!("ready to receive messages");
        server.run().await
    });

    if let Some(endpoints_server) = endpoints_server {
        endpoints_tracker.spawn(async move {
            if let Err(err) = endpoints_server.run().await {
                error!(error = ?err, "error running veritech endpoints server");
            }
        });
    }

    shutdown::graceful()
        .group(main_tracker, main_token)
        .group(endpoints_tracker, endpoints_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(graceful_shutdown_timeout)
        .wait()
        .await
        .map_err(Into::into)
}
