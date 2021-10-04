use clap::{AppSettings, Clap};

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// TODO(fnichol): fill in short help
///
/// TODO(fnichol): fill in long help
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
}
