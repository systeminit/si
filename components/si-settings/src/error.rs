use config;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SettingsError>;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("cannot initialize sodium oxide")]
    SodiumOxideInit,
    #[error("error deserializing the configuration")]
    ConfigError(#[from] config::ConfigError),
    #[error("required settings value for: {0}")]
    Required(&'static str),
}
