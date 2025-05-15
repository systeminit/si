mod api_error;
mod app_state;
mod config;
mod profiles;
mod artifacts;
mod routes;
mod server;

use thiserror::Error;

type Error = ServerError;
type Result<T> = std::result::Result<T, ServerError>;
type ServerResult<T> = std::result::Result<T, ServerError>;

/// An error than can be returned when a Rebaser service is running.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    /// When a NATS client fails to be created successfully
    #[error("nats error: {0}")]
    Nats(#[from] si_data_nats::NatsError),
}

pub use self::{
    config::{
        Config,
        ConfigError,
        ConfigFile,
        StandardConfigFile,
        detect_and_configure_development,
    },
    server::Server,
};
