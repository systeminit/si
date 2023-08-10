use color_eyre::Result;
use thiserror::Error;

pub mod cmd;
mod containers;
mod key_management;
pub mod state;

pub const CONTAINER_NAMES: &[&str] = &[
    "jaeger", "postgres", "nats", "otelcol", "council", "veritech", "pinga", "sdf", "web",
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
    #[error("join: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("Unable to find local data dir. Expected format `$HOME/.local/share` or `$HOME/Library/Application Support`")]
    MissingDataDir(),
    #[error("reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("unable to download update, status = {0}")]
    UnableToDownloadUpdate(u16),
    #[error("unable to fetch containers update, status = {0}")]
    UnableToFetchContainersUpdate(u16),
    #[error("unable to fetch si update, status = {0}")]
    UnableToFetchSiUpdate(u16),
}

pub type CliResult<T> = Result<T, SiCliError>;
