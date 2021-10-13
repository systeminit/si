use std::{convert::TryFrom, net::SocketAddr, path::PathBuf};

use clap::{AppSettings, ArgSettings, Clap};
use si_cyclone::server::{Config, ConfigError, IncomingStream};

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

    /// Path to the lang server program.
    #[clap(long, env = "SI_LANG_SERVER", setting = ArgSettings::HideEnvValues)]
    pub(crate) lang_server: PathBuf,
}

impl TryFrom<Args> for Config {
    type Error = ConfigError;

    fn try_from(args: Args) -> Result<Self, Self::Error> {
        let mut builder = Self::builder();

        if let Some(socket_addr) = args.bind_addr {
            builder.incoming_stream(IncomingStream::HTTPSocket(socket_addr));
        }
        if let Some(pathbuf) = args.bind_uds {
            builder.incoming_stream(IncomingStream::UnixDomainSocket(pathbuf));
        }

        if args.enable_ping {
            builder.enable_ping(true);
        } else if args.disable_ping {
            builder.enable_ping(false);
        }

        if args.enable_resolver {
            builder.enable_resolver(true);
        } else if args.disable_resolver {
            builder.enable_resolver(false);
        }

        if args.lang_server.is_file() {
            builder.lang_server_path(args.lang_server);
        } else {
            return Err(ConfigError::LangServerProgramNotFound(args.lang_server));
        }

        builder.build().map_err(From::from)
    }
}
