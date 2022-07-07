use color_eyre::Result;
use modat_server::{Config, Server};
use telemetry::{start_tracing_level_signal_handler_task, tracing::debug, TelemetryClient};

mod args;

const RT_DEFAULT_THREAD_STACK_SIZE: usize = 2 * 1024 * 1024 * 3;

fn main() -> Result<()> {
    tokio::runtime::Builder::new_multi_thread()
        .thread_stack_size(RT_DEFAULT_THREAD_STACK_SIZE)
        .enable_all()
        .build()?
        .block_on(async_main())
}

async fn async_main() -> Result<()> {
    color_eyre::install()?;
    let config = telemetry::Config::builder()
        .service_name("modat")
        .service_namespace("si")
        .app_modules(vec!["modat_cli", "modat"])
        .build()?;
    let telemetry = telemetry::init(config)?;
    let args = args::parse();

    run(args, telemetry).await
}

async fn run(args: args::Args, mut telemetry: telemetry::Client) -> Result<()> {
    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");

    Server::init()?;

    // TODO(fnichol): we have a mutex poisoning panic that happens, but is avoided if opentelemetry
    // is not running when the migrations are. For the moment we'll disable otel until after the
    // migrations, which means we miss out on some good migration telemetry in honeycomb, but the
    // service boots??
    //
    // See: https://app.shortcut.com/systeminit/story/1934/sdf-mutex-poison-panic-on-launch-with-opentelemetry-exporter
    let _disable_opentelemetry = args.disable_opentelemetry;
    telemetry.disable_opentelemetry().await?;
    // if args.disable_opentelemetry {
    //     telemetry.disable_opentelemetry().await?;
    // }

    let config = Config::try_from(args)?;

    let encryption_key = Server::load_encryption_key(config.cyclone_encryption_key_path()).await?;

    let nats = Server::connect_to_nats(config.nats()).await?;

    let pg_pool = Server::create_pg_pool(config.pg_pool()).await?;

    let veritech = Server::create_veritech_client(nats.clone());

    // TODO(fnichol): re-enable, which we shouldn't need in the long run
    //if !disable_opentelemetry {
    //    telemetry.enable_opentelemetry().await?;
    //}

    start_tracing_level_signal_handler_task(&telemetry)?;

    Server::uds(config, telemetry, pg_pool, nats, veritech, encryption_key)
        .await?
        .run()
        .await?;

    Ok(())
}
