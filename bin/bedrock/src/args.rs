use std::path::PathBuf;

use bedrock_server::{
    Config,
    ConfigError,
    ConfigFile,
    StandardConfigFile,
};
use clap::{
    ArgAction,
    Parser,
};
use si_service::prelude::*;

const NAME: &str = "bedrock";

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

    /// The address and port to bind the HTTP server to [example: 0.0.0.0:80]
    #[arg(long, env)]
    pub(crate) socket_addr: Option<String>,

    // Section for NATS configuration
    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<SensitiveString>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_path: Option<PathBuf>,

    /// AWS Credentials
    #[arg(long, env = "AWS_SECRET_ACCESS_KEY", hide_env_values(true))]
    aws_secret_access_key: Option<String>,
    #[arg(long, env = "AWS_ACCESS_KEY_ID", hide_env_values(true))]
    aws_access_key_id: Option<String>,
    #[arg(long, env = "AWS_SESSION_TOKEN", hide_env_values(true))]
    aws_session_token: Option<String>,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            // For the application itself
            if let Some(socket_addr) = args.socket_addr {
                config_map.set("socket_addr", socket_addr);
            }
            // For NATS
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url.clone());
            }
            if let Some(creds) = args.nats_creds {
                config_map.set("nats.creds", creds.to_string());
            }
            if let Some(creds_path) = args.nats_creds_path {
                config_map.set("nats.creds_file", creds_path.display().to_string());
            }
            // For AWS
            if let Some(aws_secret_access_key) = args.aws_secret_access_key {
                config_map.set("aws_secret_access_key", aws_secret_access_key.to_string());
            }
            if let Some(aws_access_key_id) = args.aws_access_key_id {
                config_map.set("aws_access_key_id", aws_access_key_id.to_string());
            }
            if let Some(aws_session_token) = args.aws_session_token {
                config_map.set("aws_session_token", aws_session_token.to_string());
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
