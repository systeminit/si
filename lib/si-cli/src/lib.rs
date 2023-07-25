use color_eyre::Result;
use thiserror::Error;

pub mod cmd;
mod containers;

pub const CONTAINER_NAMES: &[&str] = &[
    "jaeger", "otelcol", "postgres",
    "nats",
    // "sdf",
    // "council",
    // "veritech",
    // "pinga",
    // "web",
];

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SiCliError {
    #[error("docker api: {0}")]
    Docker(#[from] docker_api::Error),
    #[error("container search failed: {0}")]
    DockerContainerSearch(String),
    #[error("unable to connect to the docker engine")]
    DockerEngine,
    #[error("failed to launch web url {0}")]
    FailToLaunch(String),
    #[error("incorrect installation type {0}")]
    IncorrectInstallMode(String),
    #[error("aborting installation")]
    Installation,
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("unable to download update, status = {0}")]
    UnableToDownloadUpdate(u16),
}

pub type CliResult<T> = Result<T, SiCliError>;
