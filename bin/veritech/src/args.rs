use clap::{ArgAction, Parser};
use veritech::{
    server::{Config, ConfigError},
    ConfigFile, StandardConfigFile,
};

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
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// NATS connection URL [example: 0.0.0.0:4222]
    #[arg(long, short = 'u')]
    pub(crate) nats_url: Option<String>,

    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        ConfigFile::layered_load(NAME, move |config_map| {
            if let Some(url) = args.nats_url {
                config_map.set("nats.url", url);
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
