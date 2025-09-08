use std::{
    net::SocketAddr,
    path::PathBuf,
    time::Duration,
};

use clap::{
    ArgAction,
    Parser,
};
use cyclone_server::{
    Config,
    ConfigError,
    IncomingStream,
};

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
    /// Multiple -v options increase verbosity. The maximum is 6.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Disables ANSI coloring in log output, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "no-color",
        default_value = "false",
        env = "SI_NO_COLOR",
        hide_env_values = true,
        conflicts_with = "force_color"
    )]
    pub(crate) no_color: bool,

    /// Forces ANSI coloring, even if standard output refers to a terminal/TTY.
    ///
    /// For more details, visit: <http://no-color.org/>.
    #[arg(
        long = "force-color",
        default_value = "false",
        env = "SI_FORCE_COLOR",
        hide_env_values = true,
        conflicts_with = "no_color"
    )]
    pub(crate) force_color: bool,

    /// Prints telemetry logging as JSON lines.
    ///
    /// For more details, visit: <https://jsonlines.org/>.
    #[arg(
        long = "log-json",
        default_value = "false",
        env = "SI_LOG_JSON",
        hide_env_values = true
    )]
    pub(crate) log_json: bool,

    /// Additionally appends logging to rolling files under the given directory.
    #[arg(
        long = "log-file-directory",
        env = "SI_LOG_FILE_DIRECTORY",
        hide_env_values = true
    )]
    pub(crate) log_file_directory: Option<PathBuf>,

    /// Enables support for emitting async runtime data to `tokio-console`.
    ///
    /// For more details, visit: <https://github.com/tokio-rs/console>.
    #[arg(
        long = "tokio-console",
        default_value = "false",
        env = "SI_TOKIO_CONSOLE",
        hide_env_values = true
    )]
    pub(crate) tokio_console: bool,

    /// Binds service to a socket address [example: 0.0.0.0:5157]
    #[arg(long, group = "bind")]
    pub(crate) bind_addr: Option<SocketAddr>,

    /// Binds service to a unix domain socket [example: /var/run/cyclone.sock]
    #[arg(long, group = "bind")]
    pub(crate) bind_uds: Option<PathBuf>,

    /// Binds service to a vsock socket [example: 3:52]
    #[arg(long, group = "bind")]
    pub(crate) bind_vsock: Option<String>,

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

    /// Enables command run endpoint.
    #[arg(long, group = "action_run")]
    pub(crate) enable_action_run: bool,

    /// Disables command run endpoint.
    #[arg(long, group = "action_run")]
    pub(crate) disable_action_run: bool,

    /// Enables configuration endpoint.
    #[arg(long, group = "configuration")]
    pub(crate) enable_configuration: bool,

    /// Disables configuration endpoint.
    #[arg(long, group = "configuration")]
    pub(crate) disable_configuration: bool,

    /// Enables remote shell endpoint.
    #[arg(long, group = "remote_shell")]
    pub(crate) enable_remote_shell: bool,

    /// Disables remote shell endpoint.
    #[arg(long, group = "remote_shell")]
    pub(crate) disable_remote_shell: bool,

    /// Enables management endpoint.
    #[arg(long, group = "management")]
    pub(crate) enable_management: bool,

    /// Disables management endpoint.
    #[arg(long, group = "management")]
    pub(crate) disable_management: bool,

    /// Enables validation endpoint.
    #[arg(long, group = "validation")]
    pub(crate) enable_validation: bool,

    /// Disables validation endpoint.
    #[arg(long, group = "validation")]
    pub(crate) disable_validation: bool,

    /// Path to the lang server program.
    #[arg(long, env = "SI_LANG_SERVER", hide_env = true)]
    pub(crate) lang_server: PathBuf,

    /// Overrides the default function timeout of the lang server program.
    #[arg(long)]
    pub(crate) lang_server_function_timeout: Option<usize>,

    /// Limits execution requests to 1 before shutting down
    #[arg(long, group = "request_limiting")]
    pub(crate) oneshot: bool,

    /// Limits execution requests to the given value before shutting down
    #[arg(long, group = "request_limiting")]
    pub(crate) limit_requests: Option<u32>,

    /// Enables vsock forwarder.
    #[arg(long, group = "forwarder")]
    pub(crate) enable_forwarder: bool,

    /// Disables vsock forwarder.
    #[arg(long, group = "forwarder")]
    pub(crate) disable_forwarder: bool,

    /// Enables process gatherer.
    #[arg(long, group = "gatherer")]
    pub(crate) enable_process_gatherer: bool,

    /// Disables process gatherer.
    #[arg(long, group = "gatherer")]
    pub(crate) disable_process_gatherer: bool,
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

        #[cfg(target_os = "linux")]
        if let Some(addr) = args.bind_vsock {
            // todo(scott): check the format before attempting to parse
            let split = addr.split(':').collect::<Vec<&str>>();

            let vsock_addr = cyclone_server::VsockAddr::new(
                split[0].parse::<u32>().unwrap(),
                split[1].parse::<u32>().unwrap(),
            );
            builder.incoming_stream(IncomingStream::VsockSocket(vsock_addr));
        }

        builder.try_lang_server_path(args.lang_server)?;
        builder.lang_server_function_timeout(args.lang_server_function_timeout);

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

        if args.enable_action_run {
            builder.enable_action_run(true);
        } else if args.disable_action_run {
            builder.enable_action_run(false);
        }

        if args.enable_configuration {
            builder.enable_schema_variant_definition(true);
        } else if args.disable_configuration {
            builder.enable_schema_variant_definition(false);
        }

        if args.enable_remote_shell {
            builder.enable_remote_shell(true);
        } else if args.disable_remote_shell {
            builder.enable_remote_shell(false);
        }

        if args.enable_management {
            builder.enable_management(true);
        } else if args.disable_management {
            builder.enable_management(false);
        }

        if args.enable_validation {
            builder.enable_validation(true);
        } else if args.disable_validation {
            builder.enable_validation(false);
        }

        if args.oneshot {
            builder.limit_requests(1);
        } else if let Some(limit_requests) = args.limit_requests {
            builder.limit_requests(limit_requests);
        }

        if args.enable_forwarder {
            builder.enable_forwarder(true);
        } else if args.disable_forwarder {
            builder.enable_forwarder(false);
        }

        if args.enable_process_gatherer {
            builder.enable_process_gatherer(true);
        } else if args.disable_process_gatherer {
            builder.enable_forwarder(false);
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
