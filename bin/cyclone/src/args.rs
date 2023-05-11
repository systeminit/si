use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::{ArgAction, Parser};
use cyclone_server::{Config, ConfigError, IncomingStream};

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
#[command(name = NAME, max_term_width = 100)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct Args {
    /// Sets the verbosity mode.
    ///
    /// Multiple -v options increase verbosity. The maximum is 4.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Disable OpenTelemetry on startup
    #[arg(long)]
    pub(crate) disable_opentelemetry: bool,

    /// Binds service to a socket address [example: 0.0.0.0:5157]
    #[arg(long, group = "bind")]
    pub(crate) bind_addr: Option<SocketAddr>,

    /// Binds service to a unix domain socket [example: /var/run/cyclone.sock]
    #[arg(long, group = "bind")]
    pub(crate) bind_uds: Option<PathBuf>,

    /// Enables active/watch behavior.
    #[arg(long, group = "watch")]
    pub(crate) enable_watch: bool,

    /// Disables active/watch behavior.
    #[arg(long, group = "watch")]
    pub(crate) disable_watch: bool,

    /// Active/watch timeout in seconds.
    #[arg(long, default_value = "10")]
    pub(crate) watch_timeout: u64,

    /// Enables ping endpoint.
    #[arg(long, group = "ping")]
    pub(crate) enable_ping: bool,

    /// Disables ping endpoint.
    #[arg(long, group = "ping")]
    pub(crate) disable_ping: bool,

    /// Enables resolver endpoint.
    #[arg(long, group = "resolver")]
    pub(crate) enable_resolver: bool,

    /// Disables resolver endpoint.
    #[arg(long, group = "resolver")]
    pub(crate) disable_resolver: bool,

    /// Enables workflow endpoint.
    #[arg(long, group = "workflow")]
    pub(crate) enable_workflow: bool,

    /// Disables workflow endpoint.
    #[arg(long, group = "workflow")]
    pub(crate) disable_workflow: bool,

    /// Enables command run endpoint.
    #[arg(long, group = "command_run")]
    pub(crate) enable_command_run: bool,

    /// Disables command run endpoint.
    #[arg(long, group = "command_run")]
    pub(crate) disable_command_run: bool,

    /// Enables reconciliation endpoint.
    #[arg(long, group = "reconciliation")]
    pub(crate) enable_reconciliation: bool,

    /// Disables reconciliation endpoint.
    #[arg(long, group = "reconciliation")]
    pub(crate) disable_reconciliation: bool,

    /// Enables configuration endpoint.
    #[arg(long, group = "configuration")]
    pub(crate) enable_configuration: bool,

    /// Disables configuration endpoint.
    #[arg(long, group = "configuration")]
    pub(crate) disable_configuration: bool,

    /// Path to the lang server program.
    #[arg(long, env = "SI_LANG_SERVER", hide_env = true)]
    pub(crate) lang_server: PathBuf,

    /// Limits execution requests to 1 before shutting down
    #[arg(long, group = "request_limiting")]
    pub(crate) oneshot: bool,

    /// Limits execution requests to the given value before shutting down
    #[arg(long, group = "request_limiting")]
    pub(crate) limit_requests: Option<u32>,

    /// Cyclone decryption key file location [example: /run/cyclone/cyclone.key]
    #[arg(long)]
    pub(crate) decryption_key: PathBuf,
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

        builder.try_lang_server_path(args.lang_server)?;

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

        if args.oneshot {
            builder.limit_requests(1);
        } else if let Some(limit_requests) = args.limit_requests {
            builder.limit_requests(limit_requests);
        }

        builder.build().map_err(Into::into)
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
