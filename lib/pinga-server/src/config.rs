use std::path::Path;

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_faktory::FaktoryConfig;
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use telemetry::prelude::*;
use thiserror::Error;

pub use dal::CycloneKeyPair;
pub use si_settings::{StandardConfig, StandardConfigFile};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "FaktoryConfig::default()")]
    faktory: FaktoryConfig,

    cyclone_encryption_key_path: CanonicalFile,
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

    /// Gets a reference to the config's faktory.
    #[must_use]
    pub fn faktory(&self) -> &FaktoryConfig {
        &self.faktory
    }

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn cyclone_encryption_key_path(&self) -> &Path {
        self.cyclone_encryption_key_path.as_path()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pg: PgPoolConfig,
    nats: NatsConfig,
    faktory: FaktoryConfig,
    cyclone_encryption_key_path: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        let mut cyclone_encryption_key_path = "/run/pinga/cyclone_encryption.key".to_string();

        // TODO(fnichol): okay, this goes away/changes when we determine where the key would be by
        // default, etc.
        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            // In development we just take the keys cyclone is using (it needs both public and secret)
            // The dal integration tests will also need it
            cyclone_encryption_key_path = Path::new(&dir)
                .join("../../lib/cyclone-server/src/dev.encryption.key")
                .to_string_lossy()
                .to_string();
            warn!(
                cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
                "detected cargo run, setting *default* key paths from sources"
            );
        }

        Self {
            pg: Default::default(),
            nats: Default::default(),
            faktory: Default::default(),
            cyclone_encryption_key_path,
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
        config.pg_pool(value.pg);
        config.nats(value.nats);
        config.faktory(value.faktory);
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.build().map_err(Into::into)
    }
}
