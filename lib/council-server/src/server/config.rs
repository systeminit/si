use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
pub use si_settings::{StandardConfig, StandardConfigFile};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
}

pub type Result<T, E = ConfigError> = std::result::Result<T, E>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConfigFile {
    nats: NatsConfig,
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self> {
        let mut config = Config::builder();
        config.nats(value.nats);
        config.build().map_err(Into::into)
    }
}

impl Config {
    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn nats(&self) -> &NatsConfig {
        &self.nats
    }

    /// Gets a reference to the config's subject prefix.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.nats.subject_prefix.as_deref()
    }
}
