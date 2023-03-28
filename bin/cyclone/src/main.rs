use color_eyre::Result;
use cyclone_server::{Config, IncomingStream, Server};
use telemetry_application::{
    prelude::*, start_tracing_level_signal_handler_task, ApplicationTelemetryClient,
    TelemetryClient, TelemetryConfig,
};

mod args;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let config = TelemetryConfig::builder()
        .service_name("cyclone")
        .service_namespace("si")
        .log_env_var_prefix("SI")
        .app_modules(vec!["cyclone", "cyclone_server"])
        .build()?;
    let telemetry = telemetry_application::init(config)?;
    let args = args::parse();

    run(args, telemetry).await
}

async fn run(args: args::Args, mut telemetry: ApplicationTelemetryClient) -> Result<()> {
    if args.verbose > 0 {
        telemetry.set_verbosity(args.verbose.into()).await?;
    }
    debug!(arguments =?args, "parsed cli arguments");
    if args.disable_opentelemetry {
        telemetry.disable_opentelemetry().await?;
    }

    let decryption_key = Server::load_decryption_key(&args.decryption_key).await?;

    let config = Config::try_from(args)?;

    start_tracing_level_signal_handler_task(&telemetry)?;

    let telemetry = Box::new(telemetry);

    match config.incoming_stream() {
        IncomingStream::HTTPSocket(_) => {
            Server::http(config, telemetry, decryption_key)?
                .run()
                .await?
        }
        IncomingStream::UnixDomainSocket(_) => {
            Server::uds(config, telemetry, decryption_key)
                .await?
                .run()
                .await?
        }
    }

    Ok(())
}
