use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

pub(crate) use si_settings::StandardConfig;

pub use si_settings::StandardConfigFile;

const DEFAULT_CONCURRENCY_LIMIT: usize = 1000;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder error: {0}")]
    ConfigBuilder(#[from] ConfigBuilderError),
    #[error("si settings error: {0}")]
    SiSettings(#[from] si_settings::SettingsError),
}

type Result<T> = std::result::Result<T, ConfigError>;

/// The config for the forklift server.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_concurrency_limit()")]
    concurrency_limit: usize,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "default_data_warehouse_stream_name()")]
    data_warehouse_stream_name: Option<String>,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets the config's concurrency limit.
    pub fn concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn nats(&self) -> &NatsConfig {
        &self.nats
    }

    /// Gets a reference to the config's subject prefix.
    pub fn subject_prefix(&self) -> Option<&str> {
        self.nats.subject_prefix.as_deref()
    }

    /// Gets a reference to the (optional) data warehouse stream name.
    pub fn data_warehouse_stream_name(&self) -> Option<&str> {
        self.data_warehouse_stream_name.as_deref()
    }
}

#[allow(missing_docs)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default = "default_concurrency_limit")]
    concurrency_limit: usize,
    #[serde(default)]
    pub nats: NatsConfig,
    #[serde(default = "default_data_warehouse_stream_name")]
    pub data_warehouse_stream_name: Option<String>,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            instance_id: random_instance_id(),
            concurrency_limit: default_concurrency_limit(),
            nats: Default::default(),
            data_warehouse_stream_name: default_data_warehouse_stream_name(),
        }
    }
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(value: ConfigFile) -> Result<Self> {
        let mut config = Config::builder();
        config.instance_id(value.instance_id);
        config.concurrency_limit(value.concurrency_limit);
        config.nats(value.nats);
        config.data_warehouse_stream_name(value.data_warehouse_stream_name);
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_concurrency_limit() -> usize {
    DEFAULT_CONCURRENCY_LIMIT
}

fn default_data_warehouse_stream_name() -> Option<String> {
    None
}
