use std::path::PathBuf;

use clap::{ArgAction, Parser};
use si_std::SensitiveString;
use veritech_server::{Config, ConfigError, ConfigFile, StandardConfigFile};

const NAME: &str = "veritech";

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
    pub(crate) cyclone_pool_size: Option<u16>,

    /// Veritech decryption key file location [example: /run/veritech/veritech.key]
    #[arg(long)]
    pub(crate) decryption_key: Option<PathBuf>,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, move |config_map| {
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
            if let Some(decryption_key_path) = args.decryption_key {
                config_map.set(
                    "decryption_key_path",
                    decryption_key_path.display().to_string(),
                );
            }
            config_map.set("nats.connection_name", NAME);
        })?
        .try_into()
    }
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
