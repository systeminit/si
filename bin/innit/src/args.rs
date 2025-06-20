use std::path::PathBuf;

use clap::{
    ArgAction,
    Parser,
};
use innit_server::{
    Config,
    ConfigError,
    ConfigFile,
    StandardConfigFile,
};
use si_service::prelude::*;

const NAME: &str = "innit";

pub(crate) fn parse() -> Args {
    Args::parse()
}

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

    /// ARN for a Private Cert Authority in AWS
    /// Can be specified multiple times for multiple CAs
    #[arg(long, env, action = ArgAction::Append)]
    pub(crate) client_ca_arn: Vec<String>,

    /// The address and port to bind the HTTP server to [example: 0.0.0.0:80]
    #[arg(long, env)]
    pub(crate) socket_addr: Option<String>,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            if !args.client_ca_arn.is_empty() {
                config_map.set("client_ca_arns", args.client_ca_arn);
            }
            if let Some(socket_addr) = args.socket_addr {
                config_map.set("socket_addr", socket_addr);
            }
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
