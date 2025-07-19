use std::time::Duration;

use innit_client::InnitClient;
use pinga_server::Server;
use si_service::{
    color_eyre,
    prelude::*,
    rt,
    shutdown,
    startup,
    telemetry_application,
};

use crate::args::{
    NAME,
    load_config_with_provider,
};

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

const GRACEFUL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(60 * 10);

fn main() -> Result<()> {
    rt::block_on(BIN_NAME, async_main())
}

async fn async_main() -> Result<()> {
    let main_tracker = TaskTracker::new();
    let main_token = CancellationToken::new();
    let layer_db_tracker = TaskTracker::new();
    let layer_db_token = CancellationToken::new();
    let telemetry_tracker = TaskTracker::new();
    let telemetry_token = CancellationToken::new();

    color_eyre::install()?;
    let args = args::parse();
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
            .interesting_modules(vec![
                "dal",
                "naxum",
                "si_data_nats",
                "si_data_pg",
                "si_layer_cache",
                "si_service",
                "foyer_storage",
            ])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };

    startup::startup(BIN_NAME).await?;

    if args.verbose > 0 {
        telemetry
            .set_verbosity_and_wait(args.verbose.into())
            .await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    debug!("creating innit-client...");
    let provider = Some(InnitClient::new_from_environment(NAME.to_string()).await?);
    let config = load_config_with_provider(args, provider).await?;
    debug!(?config, "computed configuration");

    let server = Server::from_config(
        config,
        main_token.clone(),
        &layer_db_tracker,
        layer_db_token.clone(),
    )
    .await?;

    main_tracker.spawn(async move {
        info!("ready to receive messages");
        server.run().await
    });

    shutdown::graceful()
        .group(main_tracker, main_token)
        .group(layer_db_tracker, layer_db_token)
        .group(telemetry_tracker, telemetry_token)
        .telemetry_guard(telemetry_shutdown.into_future())
        .timeout(GRACEFUL_SHUTDOWN_TIMEOUT)
        .wait()
        .await
        .map_err(Into::into)
}
