use std::convert::TryFrom;

use clap::Parser;
use si_settings::StandardConfigFile;
use veritech::{
    server::{Config, ConfigError},
    ConfigFile,
};

const NAME: &str = "veritech";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

#[derive(Debug, Parser)]
#[clap(name = NAME, max_term_width = 100)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 3.
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: usize,

    /// NATS connection URL [example: 0.0.0.0:8080]
    #[clap(long, short = 'u')]
    pub(crate) nats_url: Option<String>,
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
