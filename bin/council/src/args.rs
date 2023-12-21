use clap::{ArgAction, Parser};

use council_server::server::config::{Config, ConfigError, ConfigFile, StandardConfigFile};

const NAME: &str = "council";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative API service.
#[derive(Parser, Debug)]
#[command(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// NATS connection URL [example: demo.nats.io]
    #[arg(long)]
    pub(crate) nats_url: Option<String>,

    /// NATS credentials string
    #[arg(long, allow_hyphen_values = true)]
    pub(crate) nats_creds: Option<String>,

    /// NATS credentials file
    #[arg(long)]
    pub(crate) nats_creds_path: Option<String>,

    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, |config_map| {
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
            }
            if let Some(creds) = args.nats_creds {
                config_map.set("nats.creds", creds);
            }
            if let Some(creds_file) = args.nats_creds_path {
                config_map.set("nats.creds_file", creds_file);
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
