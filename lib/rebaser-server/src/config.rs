use si_crypto::CryptoConfig;
use std::path::PathBuf;
use std::{env, path::Path};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use rebaser_core::RebaserMessagingConfig;
use serde::{Deserialize, Serialize};
use si_crypto::{SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_layer_cache::error::LayerDbError;
use si_std::CanonicalFileError;
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
    LayerCache(#[from] LayerDbError),
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

#[allow(missing_docs)]
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "CryptoConfig::default()")]
    crypto: CryptoConfig,

    #[builder(default = "SymmetricCryptoServiceConfig::default()")]
    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default)]
    messaging_config: RebaserMessagingConfig,

    #[builder(default = "default_layer_cache_dbname()")]
    layer_cache_pg_dbname: String,

    #[builder(default = "si_layer_cache::default_cache_path_for_service(\"rebaser\")")]
    layer_cache_disk_path: PathBuf,
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

    /// Gets a reference to the config's crypto config.
    #[must_use]
    pub fn crypto(&self) -> &CryptoConfig {
        &self.crypto
    }

    /// Gets a reference to the symmetric crypto service.
    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoServiceConfig {
        &self.symmetric_crypto_service
    }

    /// Gets a reference to the messaging config
    pub fn messaging_config(&self) -> &RebaserMessagingConfig {
        &self.messaging_config
    }

    /// Gets a reference to the layer cache's pg pool config.
    #[must_use]
    pub fn layer_cache_pg_dbname(&self) -> &str {
        &self.layer_cache_pg_dbname
    }

    /// Gets a reference to the layer cache's disk database path
    #[must_use]
    pub fn layer_cache_disk_path(&self) -> &Path {
        self.layer_cache_disk_path.as_path()
    }
}

/// The configuration file for creating a [`Server`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default = "default_layer_cache_dbname")]
    layer_cache_pg_dbname: String,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default)]
    crypto: CryptoConfig,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
    #[serde(default = "default_layer_cache_disk_path")]
    layer_cache_disk_path: PathBuf,
    #[serde(default)]
    messaging_config: RebaserMessagingConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            layer_cache_pg_dbname: default_layer_cache_dbname(),
            nats: Default::default(),
            crypto: Default::default(),
            symmetric_crypto_service: default_symmetric_crypto_config(),
            layer_cache_disk_path: default_layer_cache_disk_path(),
            messaging_config: Default::default(),
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
        config.layer_cache_pg_dbname(value.layer_cache_pg_dbname);
        config.nats(value.nats);
        config.crypto(value.crypto);
        config.symmetric_crypto_service(value.symmetric_crypto_service.try_into()?);
        config.layer_cache_disk_path(value.layer_cache_disk_path);
        config.build().map_err(Into::into)
    }
}

fn default_symmetric_crypto_config() -> SymmetricCryptoServiceConfigFile {
    SymmetricCryptoServiceConfigFile {
        active_key: None,
        active_key_base64: None,
        extra_keys: vec![],
    }
}

fn default_layer_cache_dbname() -> String {
    "si_layer_db".to_string()
}

fn default_layer_cache_disk_path() -> PathBuf {
    si_layer_cache::default_cache_path_for_service("rebaser")
}

/// This function is used to determine the development environment and update the [`ConfigFile`]
/// accordingly.
#[allow(clippy::disallowed_methods)]
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
    let symmetric_crypto_service_key = resources
        .get_ends_with("dev.donkey.key")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();
    let postgres_cert = resources
        .get_ends_with("dev.postgres.root.crt")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.crypto.encryption_key_file = cyclone_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let cyclone_encryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = Path::new(&dir)
        .join("../../lib/dal/dev.donkey.key")
        .to_string_lossy()
        .to_string();
    let postgres_cert = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.crypto.encryption_key_file = cyclone_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);

    Ok(())
}
