use std::convert::TryFrom;

use clap::{AppSettings, Clap};
use si_veritech::server::{Config, ConfigError};

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

#[derive(Clap, Debug)]
#[clap(
    global_setting = AppSettings::ColoredHelp,
    global_setting = AppSettings::UnifiedHelpMessage,
    max_term_width = 100,
)]
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
        let settings = si_settings::Settings::new()?;

        let mut builder = Self::builder();

        if let Some(nats_url) = args.nats_url {
            builder.nats_url(nats_url);
        } else {
            builder.nats_url(settings.nats.url);
        }

        builder.build().map_err(Into::into)
    }
}
