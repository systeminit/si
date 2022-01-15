use color_eyre::Result;
use telemetry::{start_tracing_level_signal_handler_task, tracing::debug, TelemetryClient};
use veritech::{Config, CycloneSpec, Server};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = telemetry::Config::builder()
        .service_name("veritech")
        .service_namespace("si")
        .app_modules(vec!["veritech_cli", "veritech"])
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
    if args.disable_opentelemetry {
        telemetry.disable_opentelemetry().await?;
    }
    let config = Config::try_from(args)?;

    start_tracing_level_signal_handler_task(&telemetry)?;

    match config.cyclone_spec() {
        CycloneSpec::LocalHttp(_) => {
            Server::for_cyclone_http(config).await?.run().await?;
        }
        CycloneSpec::LocalUds(_) => {
            Server::for_cyclone_uds(config).await?.run().await?;
        }
    }

    Ok(())
}
