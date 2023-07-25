use color_eyre::Result;
use console::Emoji;
use thiserror::Error;

pub mod cmd;
mod containers;

static PACKAGES: &[&str] = &[
    "systeminit/sdf",
    "systeminit/council",
    "systeminit/veritech",
    "systeminit/pinga",
    "systeminit/web",
    "systeminit/jaeger",
    "systeminit/otelcol",
    "systeminit/postgres",
    "systeminit/nats",
];

static STOP_COMMANDS: &[&str] = &["docker stop"];
static RESTART_COMMANDS: &[&str] = &["docker restart"];

static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", ":-)");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SiCliError {
    #[error("docker api: {0}")]
    Docker(#[from] docker_api::Error),
    #[error("unable to connect to the docker engine")]
    DockerEngine,
    #[error("failed to launch web url {0}")]
    FailToLaunch(String),
    #[error("incorrect installation type {0}")]
    IncorrectInstallMode(String),
    #[error("aborting installation")]
    Installation,
}

pub type CliResult<T> = Result<T, SiCliError>;
