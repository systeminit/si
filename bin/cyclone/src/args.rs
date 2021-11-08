use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::{ArgSettings, Parser};
use cyclone::{Config, ConfigError, IncomingStream};

const NAME: &str = "cyclone";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// A function dispatch executor.
///
/// Cyclone is a software component of the System Initiative which handles requested execution of
/// small functions in a backing language server.
#[derive(Debug, Parser)]
#[clap(name = NAME, max_term_width = 100)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[clap(short = 'v', long = "verbose", parse(from_occurrences))]
    pub(crate) verbose: usize,

    /// Binds service to a socket address [example: 0.0.0.0:5157]
    #[clap(long, group = "bind")]
    pub(crate) bind_addr: Option<SocketAddr>,

    /// Binds service to a unix domain socket [example: /var/run/cyclone.sock]
    #[clap(long, group = "bind")]
    pub(crate) bind_uds: Option<PathBuf>,

    /// Enables active/watch behavior.
    #[clap(long, group = "watch")]
    pub(crate) enable_watch: bool,

    /// Disables active/watch behavior.
    #[clap(long, group = "watch")]
    pub(crate) disable_watch: bool,

    /// Active/watch timeout in seconds.
    #[clap(long, default_value = "10")]
    pub(crate) watch_timeout: u64,

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

    /// Limits execution requests to 1 before shutting down
    #[clap(long, group = "limit_requests")]
    pub(crate) oneshot: bool,

    /// Limits execution requests to the given value before shutting down
    #[clap(long, group = "limit_requests")]
    pub(crate) limit_requests: Option<u32>,
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

        if args.enable_watch {
            builder.watch(Some(Duration::from_secs(args.watch_timeout)));
        } else if args.disable_watch {
            builder.watch(None);
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

        if args.oneshot {
            builder.limit_requests(1);
        } else if let Some(limit_requests) = args.limit_requests {
            builder.limit_requests(limit_requests);
        }

        builder.build().map_err(Into::into)
    }
}
