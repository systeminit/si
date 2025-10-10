use std::{
    env,
    path::Path,
};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_crypto::{
    SymmetricCryptoServiceConfig,
    SymmetricCryptoServiceConfigFile,
    VeritechCryptoConfig,
};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_layer_cache::{
    db::LayerDbConfig,
    error::LayerDbError,
};
pub use si_settings::{
    StandardConfig,
    StandardConfigFile,
};
use si_std::CanonicalFileError;
use si_tls::CertificateSource;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

const DEFAULT_CONCURRENCY_LIMIT: usize = 64;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("layer cache error: {0}")]
    LayerCache(#[from] LayerDbError),
    #[error("settings error: {0}")]
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

    #[builder(default = "VeritechCryptoConfig::default()")]
    crypto: VeritechCryptoConfig,

    #[builder(default = "default_concurrency_limit()")]
    concurrency_limit: usize,

    #[builder(default = "default_max_deliver()")]
    max_deliver: i64,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "SymmetricCryptoServiceConfig::default()")]
    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default = "default_layer_db_config()")]
    layer_db_config: LayerDbConfig,
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

    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoServiceConfig {
        &self.symmetric_crypto_service
    }

    /// Gets the config's concurrency limit.
    pub fn concurrency_limit(&self) -> usize {
        self.concurrency_limit
    }

    /// Gets the config's max delivery setting for NATS JetStream consumer(s).
    pub fn max_deliver(&self) -> i64 {
        self.max_deliver
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

    #[must_use]
    pub fn layer_db_config(&self) -> &LayerDbConfig {
        &self.layer_db_config
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default)]
    crypto: VeritechCryptoConfig,
    #[serde(default = "default_concurrency_limit")]
    concurrency_limit: usize,
    #[serde(default = "default_max_deliver")]
    max_deliver: i64,
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default = "default_layer_db_config")]
    layer_db_config: LayerDbConfig,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            nats: Default::default(),
            concurrency_limit: default_concurrency_limit(),
            max_deliver: default_max_deliver(),
            crypto: Default::default(),
            instance_id: random_instance_id(),
            layer_db_config: default_layer_db_config(),
            symmetric_crypto_service: default_symmetric_crypto_config(),
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
        config.concurrency_limit(value.concurrency_limit);
        config.max_deliver(value.max_deliver);
        config.instance_id(value.instance_id);
        config.symmetric_crypto_service(value.symmetric_crypto_service.try_into()?);
        config.layer_db_config(value.layer_db_config);
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

fn default_concurrency_limit() -> usize {
    DEFAULT_CONCURRENCY_LIMIT
}

fn default_max_deliver() -> i64 {
    1
}

fn default_layer_db_config() -> LayerDbConfig {
    LayerDbConfig::default()
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
    let postgres_key = resources
        .get_ends_with("dev.postgres.root.crt")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        veritech_encryption_key_path = veritech_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_key = postgres_key.as_str(),
        "detected development run",
    );

    config.crypto.encryption_key_file = veritech_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate = Some(CertificateSource::Path(postgres_key.clone().try_into()?));
    config.layer_db_config.pg_pool_config.certificate =
        Some(CertificateSource::Path(postgres_key.clone().try_into()?));
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
    let postgres_key = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();

    warn!(
        veritech_encryption_key_path = veritech_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_key = postgres_key.as_str(),
        "detected development run",
    );

    config.crypto.encryption_key_file = veritech_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate = Some(CertificateSource::Path(postgres_key.clone().try_into()?));
    config.layer_db_config.pg_pool_config.certificate =
        Some(CertificateSource::Path(postgres_key.clone().try_into()?));
    config.layer_db_config.pg_pool_config.dbname = "si_layer_db".to_string();

    Ok(())
}
