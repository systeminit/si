use std::{net::SocketAddr, path::PathBuf};

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

    /// Binds service to a socket address [example: 0.0.0.0:8080]
    #[clap(long, group = "bind")]
    pub(crate) bind_addr: Option<SocketAddr>,

    /// Binds service to a unix domain socket [example: /var/run/cyclone.sock]
    #[clap(long, group = "bind")]
    pub(crate) bind_uds: Option<PathBuf>,

    /// Enables ping endpoint.
    #[clap(long, group = "ping")]
    pub(crate) enable_ping: bool,

    /// Disables ping endpoint.
    #[clap(long, group = "ping")]
    pub(crate) disable_ping: bool,

    /// Enables resolver endpoint.
    #[clap(long, group = "resolver")]
    pub(crate) enable_resolver: bool,

    /// Disables resolver endpoint.
    #[clap(long, group = "resolver")]
    pub(crate) disable_resolver: bool,
}
