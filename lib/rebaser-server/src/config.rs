use si_crypto::VeritechCryptoConfig;
use si_layer_cache::db::LayerDbConfig;
use std::{env, path::Path};
use ulid::Ulid;

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

    #[builder(default = "VeritechCryptoConfig::default()")]
    crypto: VeritechCryptoConfig,

    #[builder(default = "SymmetricCryptoServiceConfig::default()")]
    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default)]
    messaging_config: RebaserMessagingConfig,

    #[builder(default = "default_layer_db_config()")]
    layer_db_config: LayerDbConfig,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "5000")]
    dvu_interval_millis: u64,
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
    pub fn crypto(&self) -> &VeritechCryptoConfig {
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

    /// Gets a reference to the layderdb config
    #[must_use]
    pub fn layer_db_config(&self) -> &LayerDbConfig {
        &self.layer_db_config
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

    /// Gets the duration of the dvu interval
    pub fn dvu_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.dvu_interval_millis)
    }
}

/// The configuration file for creating a [`Server`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default)]
    crypto: VeritechCryptoConfig,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
    #[serde(default)]
    messaging_config: RebaserMessagingConfig,
    #[serde(default = "default_layer_db_config")]
    layer_db_config: LayerDbConfig,
    #[serde(default = "random_instance_id")]
    instance_id: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            nats: Default::default(),
            crypto: Default::default(),
            symmetric_crypto_service: default_symmetric_crypto_config(),
            layer_db_config: default_layer_db_config(),
            messaging_config: Default::default(),
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
        config.crypto(value.crypto);
        config.symmetric_crypto_service(value.symmetric_crypto_service.try_into()?);
        config.layer_db_config(value.layer_db_config);
        config.instance_id(value.instance_id);
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_symmetric_crypto_config() -> SymmetricCryptoServiceConfigFile {
    SymmetricCryptoServiceConfigFile {
        active_key: None,
        active_key_base64: None,
        extra_keys: vec![],
    }
}

fn default_layer_db_config() -> LayerDbConfig {
    LayerDbConfig::default_for_service("rebaser")
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

    let veritech_encryption_key_path = resources
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
        veritech_encryption_key_path = veritech_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.crypto.encryption_key_file = veritech_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.certificate_path =
        Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.dbname = "si_layer_db".to_string();
    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let veritech_encryption_key_path = Path::new(&dir)
        .join("../../lib/veritech-server/src/dev.encryption.key")
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
        veritech_encryption_key_path = veritech_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.crypto.encryption_key_file = veritech_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.certificate_path =
        Some(postgres_cert.clone().try_into()?);
    config.layer_db_config.pg_pool_config.dbname = "si_layer_db".to_string();

    Ok(())
}
