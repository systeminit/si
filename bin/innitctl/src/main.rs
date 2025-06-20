use args::{
    NAME,
    load_config_with_provider,
};
use innit_client::InnitClient;
use innitctl_backend::templatize;
use si_service::{
    color_eyre,
    prelude::*,
};

mod args;

const BIN_NAME: &str = env!("CARGO_BIN_NAME");
const LIB_NAME: &str = concat!(env!("CARGO_BIN_NAME"), "_server");

#[tokio::main]
async fn main() -> Result<()> {
    let telemetry_tracker = TaskTracker::new();
    let telemetry_token = CancellationToken::new();
    color_eyre::install()?;
    let args = args::parse();
    let (mut telemetry, _telemetry_shutdown) = {
        let config = TelemetryConfig::builder()
            .force_color(args.force_color.then_some(true))
            .no_color(args.no_color.then_some(true))
            .log_format(args.log_json.then_some(LogFormat::Json).unwrap_or_default())
            .log_file_directory(args.log_file_directory.clone())
            .service_name(BIN_NAME)
            .service_namespace("si")
            .log_env_var_prefix("SI")
            .app_modules(vec![BIN_NAME, LIB_NAME])
            .build()?;

        telemetry_application::init(config, &telemetry_tracker, telemetry_token.clone())?
    };
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

    templatize(&config).await?;
    Ok(())
}
