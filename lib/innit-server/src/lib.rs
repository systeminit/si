use thiserror::Error;

mod api_error;
mod app_state;
mod config;
mod routes;
mod server;

pub use self::{
    config::{
        Config, ConfigError, ConfigFile, StandardConfigFile, detect_and_configure_development,
    },
    server::Server,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ServerError {}
