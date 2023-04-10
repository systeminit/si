use std::path::Path;

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

const CONCURRENCY_DEFAULT: usize = 50;

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

    cyclone_encryption_key_path: CanonicalFile,

    #[builder(default = "CONCURRENCY_DEFAULT")]
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
    pg: PgPoolConfig,
    nats: NatsConfig,
    cyclone_encryption_key_path: String,
    concurrency_limit: usize,
    instance_id: String,
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
            cyclone_encryption_key_path,
            concurrency_limit: CONCURRENCY_DEFAULT,
            instance_id: random_instance_id(),
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
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.instance_id(value.instance_id);
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}
