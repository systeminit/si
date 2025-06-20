use std::path::PathBuf;

use clap::{
    ArgAction,
    Parser,
};
use forklift_server::{
    Config,
    ConfigError,
    ConfigFile,
    ConfigMap,
    ParameterProvider,
    StandardConfigFile,
};
use si_service::prelude::*;

pub const NAME: &str = "forklift";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// Arguments for the forklift service.
#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 6.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Disables ANSI coloring in log output, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "no-color",
        default_value = "false",
        env = "SI_NO_COLOR",
        hide_env_values = true,
        conflicts_with = "force_color"
    )]
    pub(crate) no_color: bool,

    /// Forces ANSI coloring, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "force-color",
        default_value = "false",
        env = "SI_FORCE_COLOR",
        hide_env_values = true,
        conflicts_with = "no_color"
    )]
    pub(crate) force_color: bool,

    /// Prints telemetry logging as JSON lines.
    ///
    /// For more details, visit: <https://jsonlines.org/>.
    #[arg(
        long = "log-json",
        default_value = "false",
        env = "SI_LOG_JSON",
        hide_env_values = true
    )]
    pub(crate) log_json: bool,

    /// Additionally appends logging to rolling files under the given directory.
    #[arg(
        long = "log-file-directory",
        env = "SI_LOG_FILE_DIRECTORY",
        hide_env_values = true
    )]
    pub(crate) log_file_directory: Option<PathBuf>,

    /// Enables support for emitting async runtime data to `tokio-console`.
    ///
    /// For more details, visit: <https://github.com/tokio-rs/console>.
    #[arg(
        long = "tokio-console",
        default_value = "false",
        env = "SI_TOKIO_CONSOLE",
        hide_env_values = true
    )]
    pub(crate) tokio_console: bool,

    /// The ID of this forklift instance [example: 01GWEAANW5BVFK5KDRVS6DEY0F"]
    #[arg(long)]
    pub(crate) instance_id: Option<String>,

    /// Overrides the default number of concurrent requests that can be processed
    #[arg(long)]
    pub(crate) concurrency_limit: Option<u32>,

    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<SensitiveString>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_path: Option<PathBuf>,

    /// The name of the data warehouse stream
    #[arg(long)]
    pub(crate) data_warehouse_stream_name: Option<String>,

    /// Enables the audit logs app
    #[arg(long)]
    pub(crate) enable_audit_logs_app: Option<bool>,
}

fn build_config_map(args: Args, config_map: &mut ConfigMap) -> &ConfigMap {
    if let Some(instance_id) = args.instance_id {
        config_map.set("instance_id", instance_id);
    }
    if let Some(concurrency_limit) = args.concurrency_limit {
        config_map.set("concurrency_limit", i64::from(concurrency_limit));
    }
    if let Some(url) = args.nats_url {
        config_map.set("nats.url", url.clone());
    }
    if let Some(creds) = args.nats_creds {
        config_map.set("nats.creds", creds.to_string());
    }
    if let Some(creds_path) = args.nats_creds_path {
        config_map.set("nats.creds_file", creds_path.display().to_string());
    }
    config_map.set("nats.connection_name", NAME);
    if let Some(data_warehouse_stream_name) = args.data_warehouse_stream_name {
        config_map.set("data_warehouse_stream_name", data_warehouse_stream_name);
    }
    if let Some(enable_audit_logs_app) = args.enable_audit_logs_app {
        config_map.set("enable_audit_logs_app", enable_audit_logs_app);
    }
    config_map
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            build_config_map(args, config_map);
        })?
        .try_into()
    }
}

pub async fn load_config_with_provider<P>(
    args: Args,
    provider: Option<P>,
) -> Result<Config, ConfigError>
where
    P: ParameterProvider + 'static,
{
    ConfigFile::layered_load_with_provider::<_, P>(NAME, provider, move |config_map| {
        build_config_map(args, config_map);
    })
    .await?
    .try_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_command() {
        use clap::CommandFactory;
        Args::command().debug_assert()
    }
}
