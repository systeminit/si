use clap::{builder::PossibleValuesParser, Parser, Subcommand};
use std::str::FromStr;
use strum::{Display, EnumString, EnumVariantNames};

const NAME: &str = "si";

/// Parse, validate, and return the CLI arguments as a typed struct.
pub(crate) fn parse() -> Args {
    Args::parse()
}

/// The System Initiative Launcher.
#[derive(Debug, Parser)]
#[command(version = include_str!("version.txt"))]
#[command(
name = NAME,
about = "The System Initiative Launcher

@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@@(..................................,(@%*...............(@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                  /&@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
@@/                                                      /@@
@@/                                                      /@@
@@/                                                      /@@
@@/                                                      /@@
@@/                                                      /@@
@@/                                                      /@@
@@#/************************************,                /@@
@@&%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%&@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@/                                  .(@%,               /@@
@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@

")]
pub(crate) struct Args {
    /// The System Initiative Launcher mode
    #[arg(value_parser = PossibleValuesParser::new(Mode::variants()))]
    #[arg(long, short, env = "SI_LAUNCHER_MODE", default_value = "local")]
    mode: String,
    /// Show a preview of what the System Initiative Launcher will do
    #[arg(long, short = 'p', default_value = "false")]
    pub is_preview: bool,
    
    /// Allows starting the web service and binding to a specific IP
    #[arg(long = "web-host", env = "SI_WEB_ADDRESS", default_value = "127.0.0.1")]
    pub web_host: String,

    /// Allows starting the web service and binding to a specific port
    #[arg(long = "web-port", env = "SI_WEB_PORT", default_value = "8080")]
    pub web_port: u32,

    /// The engine in which to launch System Initiate Containers
    #[arg(value_parser = PossibleValuesParser::new(Engine::variants()))]
    #[arg(long, short, env = "SI_CONTAINER_ENGINE", default_value = "docker")]
    engine: String,
    /// A path to a docker.sock file. The default paths checked are `/var/run/docker.sock`
    /// and `$HOME/.docker/run/docker.sock"`. Passing a value here will be an explicit
    /// usage of that location.
    #[arg(long, env = "SI_DOCKER_SOCK")]
    pub docker_sock: Option<String>,
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Checks that the system is setup correctly to run System Initiative
    Check(CheckArgs),
    /// Installs the necessary components to run System Initiative
    Install(InstallArgs),
    /// Launches the System Initiative Web UI.
    Launch(LaunchArgs),
    /// Starts all of the System Initiative components
    Start(StartArgs),
    /// Configures the appropriate services needed to run System Initiative (AWS, Docker, etc.)
    Configure(ConfigureArgs),
    /// Restarts all of the System Initiative components
    Restart(RestartArgs),
    /// Stops all of the System Initiative components
    Stop(StopArgs),
    /// Deletes all of the System Initiative components
    Delete(DeleteArgs),
    /// Updates the System Initiative CLI Launcher
    Update(UpdateArgs),
    /// Checks the status of the specified installation mode
    Status(StatusArgs),
    // Reports an error to System Initiative.
    // Report(ReportArgs),
}

#[derive(Debug, clap::Args)]
pub(crate) struct LaunchArgs {
    /// Allows the launching of the metrics collection endpoint
    #[clap(long)]
    pub metrics: bool,
    
}

// #[derive(Debug, clap::Args)]
// pub(crate) struct ReportArgs {}

#[derive(Debug, clap::Args)]
pub(crate) struct ConfigureArgs {
    /// Forces the reconfiguration of System Initiative credentials.
    ///
    /// Please Note, you will need to restart the applications to apply these new credentials
    #[clap(short, long)]
    pub force_reconfigure: bool,
}

#[derive(Debug, clap::Args)]
pub(crate) struct StatusArgs {
    /// Shows the logs from the containers
    #[clap(long)]
    pub show_logs: bool,

    /// The number of log lines to show when `show_logs` is used
    #[arg(long, short = 'l', default_value = "10")]
    pub log_lines: usize,
}

#[derive(Debug, clap::Args)]
pub(crate) struct StartArgs {
}

#[derive(Debug, clap::Args)]
pub(crate) struct RestartArgs {}

#[derive(Debug, clap::Args)]
pub(crate) struct StopArgs {}

#[derive(Debug, clap::Args)]
pub(crate) struct CheckArgs {}

#[derive(Debug, clap::Args)]
pub(crate) struct DeleteArgs {
    /// Keep containers so you don't have to redownload them every time
    #[clap(long, env = "SI_KEEP_IMAGES_ON_DELETE")]
    pub keep_images: bool,
}

#[derive(Debug, clap::Args)]
pub(crate) struct UpdateArgs {
    /// Skip the confirmation check as part of the update command
    #[clap(short = 'y', long)]
    pub skip_confirmation: bool,
    /// Skip the containers update as part of the update command
    #[clap(name = "self", short, long)]
    pub binary: bool,
}

#[derive(Debug, clap::Args)]
pub(crate) struct InstallArgs {
    /// Skip the system check as part of the install command
    #[clap(long)]
    pub skip_check: bool,
}

impl Args {
    pub(crate) fn mode(&self) -> Mode {
        Mode::from_str(&self.mode).expect("mode is a validated input str")
    }

    pub(crate) fn engine(&self) -> Engine {
        Engine::from_str(&self.engine).expect("engine is a validated input str")
    }
}

#[derive(Clone, Copy, Debug, Display, EnumString, EnumVariantNames)]
pub enum Mode {
    #[strum(serialize = "local")]
    Local,
}

#[derive(Clone, Copy, Debug, Display, EnumString, EnumVariantNames)]
pub enum Engine {
    #[strum(serialize = "docker")]
    Docker,
    #[strum(serialize = "podman")]
    Podman,
}

impl Mode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }
}

impl Engine {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }
}
