use std::{env, path::Path};

use buck2_resources::Buck2Resources;
use dal::CycloneKeyPair;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::StandardConfig;
use crate::StandardConfigFile;

#[allow(missing_docs)]
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

/// The set of configuration options for building a [`Server`].
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    cyclone_encryption_key_path: CanonicalFile,

    #[builder(default = "false")]
    recreate_management_stream: bool,
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

    /// Gets the toggle on if the RabbitMQ Stream will be re-created
    pub fn recreate_management_stream(&self) -> bool {
        self.recreate_management_stream
    }
}

/// The configuration file for creating a [`Server`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default = "default_cyclone_encryption_key_path")]
    cyclone_encryption_key_path: String,
    #[serde(default = "default_recreate_management_stream")]
    recreate_management_stream: bool,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            nats: Default::default(),
            cyclone_encryption_key_path: default_cyclone_encryption_key_path(),
            recreate_management_stream: false,
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
        config.recreate_management_stream(value.recreate_management_stream);
        config.build().map_err(Into::into)
    }
}

fn default_cyclone_encryption_key_path() -> String {
    "/run/rebaser/cyclone_encryption.key".to_string()
}

fn default_recreate_management_stream() -> bool {
    false
}

#[allow(clippy::disallowed_methods)] // Used to determine if running in development
fn detect_and_configure_development(config: &mut ConfigFile) -> Result<()> {
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
