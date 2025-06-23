use std::path::PathBuf;

use clap::{
    ArgAction,
    Parser,
};
use si_service::prelude::*;
use veritech_server::{
    Config,
    ConfigError,
    ConfigFile,
    ConfigMap,
    ParameterProvider,
    StandardConfigFile,
};

pub const NAME: &str = "veritech";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

#[derive(Debug, Parser)]
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

    /// Pins main Tokio runtime to specific CPU cores [example: 0-3]
    #[arg(
        long = "tokio-cpu-cores",
        env = "SI_TOKIO_CPU_CORES",
        hide_env_values = true
    )]
    pub(crate) tokio_cpu_cores: Option<CoreIds>,

    /// NATS connection URL [example: 0.0.0.0:4222]
    #[arg(long, short = 'u')]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<SensitiveString>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_path: Option<PathBuf>,

    /// Cyclone runtime type: LocalProcess
    #[arg(long)]
    pub(crate) cyclone_local_process: bool,

    /// Cyclone runtime type: LocalDocker
    #[arg(long)]
    pub(crate) cyclone_local_docker: bool,

    /// Cyclone runtime type: LocalFirecracker
    #[arg(long)]
    pub(crate) cyclone_local_firecracker: bool,

    /// Cyclone firecracker connect timeout
    #[arg(long)]
    pub(crate) cyclone_connect_timeout: Option<u64>,

    /// Cyclone pool size
    #[arg(long)]
    pub(crate) cyclone_pool_size: Option<u32>,

    /// Cyclone create firecracker setup scripts
    #[arg(long)]
    pub(crate) cyclone_create_firecracker_setup_scripts: Option<bool>,

    /// Veritech decryption key file location [example: /run/veritech/veritech.key]
    #[arg(long)]
    pub(crate) decryption_key: Option<PathBuf>,

    /// Execution timeout when communicating with a cyclone instance executing a function
    #[arg(long)]
    pub(crate) cyclone_client_execution_timeout_secs: Option<u64>,

    /// The number of concurrent requests that will be processed
    #[arg(long)]
    pub(crate) veritech_requests_concurrency_limit: Option<u32>,

    /// Instance ID [example: 01GWEAANW5BVFK5KDRVS6DEY0F"]
    ///
    /// And instance ID is used when tracking the execution of jobs in a way that can be traced
    /// back to an instance of a Pinga service.
    #[arg(long)]
    pub(crate) instance_id: Option<String>,

    /// Overrides the default graceful shutdown timeout (in seconds).
    #[arg(long)]
    pub(crate) graceful_shutdown_timeout_secs: Option<u64>,

    /// Enables or disables the heartbeat app.
    #[arg(long)]
    pub(crate) heartbeat_app: Option<bool>,

    /// Overrides the default heartbeat app sleep duration (in seconds).
    #[arg(long)]
    pub(crate) heartbeat_app_sleep_secs: Option<u64>,

    /// Overrides the default heartbeat app publish timeout duration (in seconds).
    #[arg(long)]
    pub(crate) heartbeat_app_publish_timeout_secs: Option<u64>,
}

impl Args {
    pub fn tokio_cpu_cores(&self) -> Option<Vec<CoreId>> {
        self.tokio_cpu_cores.clone().map(|c| c.into_inner())
    }
}

fn build_config_map(args: Args, config_map: &mut ConfigMap) -> &ConfigMap {
    if let Some(url) = args.nats_url {
        config_map.set("nats.url", url);
    }
    if let Some(creds) = args.nats_creds {
        config_map.set("nats.creds", creds.to_string());
    }
    if let Some(creds_file) = args.nats_creds_path {
        config_map.set("nats.creds_file", creds_file.display().to_string());
    }

    if args.cyclone_local_firecracker {
        config_map.set("cyclone.runtime_strategy", "LocalFirecracker");
    }
    if args.cyclone_local_docker {
        config_map.set("cyclone.runtime_strategy", "LocalDocker");
    }
    if args.cyclone_local_process {
        config_map.set("cyclone.runtime_strategy", "LocalProcess");
    }
    if let Some(timeout) = args.cyclone_connect_timeout {
        config_map.set("cyclone.connect_timeout", timeout);
    }
    if let Some(size) = args.cyclone_pool_size {
        config_map.set("cyclone.pool_size", size);
    }
    if let Some(cyclone_create_firecracker_setup_scripts) =
        args.cyclone_create_firecracker_setup_scripts
    {
        config_map.set(
            "cyclone.create_firecracker_setup_scripts",
            cyclone_create_firecracker_setup_scripts,
        );
    }

    if let Some(decryption_key_path) = args.decryption_key {
        config_map.set(
            "decryption_key_path",
            decryption_key_path.display().to_string(),
        );
    }
    config_map.set("nats.connection_name", NAME);
    if let Some(timeout) = args.cyclone_client_execution_timeout_secs {
        config_map.set("cyclone_client_execution_timeout_secs", timeout);
    }
    if let Some(veritech_requests_concurrency_limit) = args.veritech_requests_concurrency_limit {
        config_map.set(
            "veritech_requests_concurrency_limit",
            i64::from(veritech_requests_concurrency_limit),
        );
    }
    if let Some(instance_id) = args.instance_id {
        config_map.set("instance_id", instance_id);
    }

    if let Some(graceful_shutdown_timeout_secs) = args.graceful_shutdown_timeout_secs {
        config_map.set(
            "graceful_shutdown_timeout_secs",
            graceful_shutdown_timeout_secs,
        );
    }

    if let Some(heartbeat_app) = args.heartbeat_app {
        config_map.set("heartbeat_app", heartbeat_app);
    }
    if let Some(heartbeat_app_sleep_secs) = args.heartbeat_app_sleep_secs {
        config_map.set("heartbeat_app_sleep_secs", heartbeat_app_sleep_secs);
    }
    if let Some(heartbeat_app_publish_timeout_secs) = args.heartbeat_app_publish_timeout_secs {
        config_map.set(
            "heartbeat_app_publish_timeout_secs",
            heartbeat_app_publish_timeout_secs,
        );
    }
    config_map
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, move |config_map| {
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
