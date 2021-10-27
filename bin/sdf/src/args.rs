use clap::{AppSettings, Clap};
use sdf::{Config, ConfigError};

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative API service.
///
/// Super Dimension Fortress (SDF) is the central and primary API surface which handles front end
/// calls and dispatches function executions, among other great things.
#[derive(Clap, Debug)]
#[clap(
    name = "sdf",
    global_setting = AppSettings::ColoredHelp,
    global_setting = AppSettings::UnifiedHelpMessage,
    max_term_width = 100,
)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: usize,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let builder = Self::builder();

        builder.build().map_err(Into::into)
    }
}
