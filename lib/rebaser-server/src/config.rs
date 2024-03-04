use std::{env, path::Path};

use buck2_resources::Buck2Resources;
use content_store::PgStoreTools;
use derive_builder::Builder;
use rebaser_core::RebaserMessagingConfig;
use serde::{Deserialize, Serialize};
use si_crypto::{SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_std::{CanonicalFile, CanonicalFileError};
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

#[allow(missing_docs)]
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    cyclone_encryption_key_path: CanonicalFile,

    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default)]
    messaging_config: RebaserMessagingConfig,

    #[builder(default = "PgStoreTools::default_pool_config()")]
    content_store_pg_pool: PgPoolConfig,
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

    /// Gets a reference to the symmetric crypto service.
    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoServiceConfig {
        &self.symmetric_crypto_service
    }

    /// Gets a reference to the messaging config
    pub fn messaging_config(&self) -> &RebaserMessagingConfig {
        &self.messaging_config
    }

    /// Gets a reference to the config's content store pg pool.
    #[must_use]
    pub fn content_store_pg_pool(&self) -> &PgPoolConfig {
        &self.content_store_pg_pool
    }
}

/// The configuration file for creating a [`Server`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default = "PgStoreTools::default_pool_config")]
    content_store_pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default = "default_cyclone_encryption_key_path")]
    cyclone_encryption_key_path: String,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
    #[serde(default)]
    messaging_config: RebaserMessagingConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            content_store_pg: PgStoreTools::default_pool_config(),
            nats: Default::default(),
            cyclone_encryption_key_path: default_cyclone_encryption_key_path(),
            symmetric_crypto_service: default_symmetric_crypto_config(),
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
        config.content_store_pg_pool(value.content_store_pg);
        config.nats(value.nats);
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.symmetric_crypto_service(value.symmetric_crypto_service.try_into()?);
        config.build().map_err(Into::into)
    }
}

fn default_cyclone_encryption_key_path() -> String {
    "/run/rebaser/cyclone_encryption.key".to_string()
}

fn default_symmetric_crypto_config() -> SymmetricCryptoServiceConfigFile {
    SymmetricCryptoServiceConfigFile {
        active_key: Some("/run/rebaser/donkey.key".to_owned()),
        active_key_base64: None,
        extra_keys: vec![],
    }
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

    config.cyclone_encryption_key_path = cyclone_encryption_key_path;
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.content_store_pg.certificate_path = Some(postgres_cert.try_into()?);

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

    config.cyclone_encryption_key_path = cyclone_encryption_key_path;
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.content_store_pg.certificate_path = Some(postgres_cert.try_into()?);

    Ok(())
}
