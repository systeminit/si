use std::{env, path::Path};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use telemetry::prelude::*;
use thiserror::Error;

pub use dal::CycloneKeyPair;
pub use si_settings::{StandardConfig, StandardConfigFile};
use ulid::Ulid;

const DEFAULT_CONCURRENCY_LIMIT: usize = 5;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    cyclone_encryption_key_path: CanonicalFile,

    #[builder(default = "default_concurrency_limit()")]
    concurrency: usize,

    #[builder(default = "random_instance_id()")]
    instance_id: String,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets a reference to the config's pg pool.
    #[must_use]
    pub fn pg_pool(&self) -> &PgPoolConfig {
        &self.pg_pool
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

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn cyclone_encryption_key_path(&self) -> &Path {
        self.cyclone_encryption_key_path.as_path()
    }

    /// Gets the config's concurrency limit.
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default = "default_cyclone_encryption_key_path")]
    cyclone_encryption_key_path: String,
    #[serde(default = "default_concurrency_limit")]
    concurrency_limit: usize,
    #[serde(default = "random_instance_id")]
    instance_id: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            nats: Default::default(),
            cyclone_encryption_key_path: default_cyclone_encryption_key_path(),
            concurrency_limit: default_concurrency_limit(),
            instance_id: random_instance_id(),
        }
    }
}

impl StandardConfigFile for ConfigFile {
    type Error = ConfigError;
}

impl TryFrom<ConfigFile> for Config {
    type Error = ConfigError;

    fn try_from(mut value: ConfigFile) -> Result<Self> {
        detect_and_configure_development(&mut value)?;

        let mut config = Config::builder();
        config.pg_pool(value.pg);
        config.nats(value.nats);
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.concurrency(value.concurrency_limit);
        config.instance_id(value.instance_id);
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_cyclone_encryption_key_path() -> String {
    "/run/gobbler/cyclone_encryption.key".to_string()
}

fn default_concurrency_limit() -> usize {
    DEFAULT_CONCURRENCY_LIMIT
}

#[allow(clippy::disallowed_methods)] // Used to determine if running in development
pub fn detect_and_configure_development(config: &mut ConfigFile) -> Result<()> {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        buck2_development(config)
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        cargo_development(dir, config)
    } else {
        Ok(())
    }
}

fn buck2_development(config: &mut ConfigFile) -> Result<()> {
    let resources = Buck2Resources::read().map_err(ConfigError::development)?;

    let cyclone_encryption_key_path = resources
        .get_ends_with("dev.encryption.key")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        "detected development run",
    );

    config.cyclone_encryption_key_path = cyclone_encryption_key_path;

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let cyclone_encryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        "detected development run",
    );

    config.cyclone_encryption_key_path = cyclone_encryption_key_path;

    Ok(())
}
