use std::{
    env,
    path::Path,
    time::Duration,
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
use si_layer_cache::db::LayerDbConfig;
pub(crate) use si_settings::StandardConfig;
pub use si_settings::StandardConfigFile;
use si_std::CanonicalFileError;
use si_tls::CertificateSource;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

const DEFAULT_CONCURRENCY_LIMIT: Option<usize> = None;
const DEFAULT_PARALLEL_BUILD_LIMIT: usize = 50;

const DEFAULT_QUIESCENT_PERIOD_SECS: u64 = 60 * 10;
const DEFAULT_QUIESCENT_PERIOD: Duration = Duration::from_secs(DEFAULT_QUIESCENT_PERIOD_SECS);

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
    LayerCache(#[from] si_layer_cache::LayerDbError),
    #[error("si settings error: {0}")]
    SiSettings(#[from] si_settings::SettingsError),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

/// The config for the forklift server.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_concurrency_limit()")]
    concurrency_limit: Option<usize>,

    #[builder(default = "default_parallel_build_limit()")]
    parallel_build_limit: usize,

    #[builder(default = "default_streaming_patches()")]
    streaming_patches: bool,

    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "VeritechCryptoConfig::default()")]
    crypto: VeritechCryptoConfig,

    #[builder(default = "SymmetricCryptoServiceConfig::default()")]
    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default = "default_layer_db_config()")]
    layer_db_config: LayerDbConfig,

    #[builder(default = "default_quiescent_period()")]
    quiescent_period: Duration,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets the config's concurrency limit.
    pub fn concurrency_limit(&self) -> Option<usize> {
        self.concurrency_limit
    }

    /// Gets the config's parallel build limit.
    pub fn parallel_build_limit(&self) -> usize {
        self.parallel_build_limit
    }

    /// Gets whether edda should stream patches, or send as a single batch.
    pub fn streaming_patches(&self) -> bool {
        self.streaming_patches
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

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

    /// Gets a reference to the layderdb config
    #[must_use]
    pub fn layer_db_config(&self) -> &LayerDbConfig {
        &self.layer_db_config
    }

    /// Gets the period of inactivity before a change set consuming stream will shut down
    pub fn quiescent_period(&self) -> Duration {
        self.quiescent_period
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "random_instance_id")]
    instance_id: String,
    // TODO(fnichol): this field is prefixed with `edda_` due to our current rendering of systemd
    // units (i.e. our global `service.toml`) which will only render *one* concurrency limit for
    // all service types
    #[serde(default = "default_concurrency_limit")]
    edda_concurrency_limit: Option<usize>,
    #[serde(default = "default_parallel_build_limit")]
    edda_parallel_build_limit: usize,
    #[serde(default = "default_streaming_patches")]
    streaming_patches: bool,
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default)]
    crypto: VeritechCryptoConfig,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
    #[serde(default = "default_layer_db_config")]
    layer_db_config: LayerDbConfig,
    #[serde(default = "default_quiescent_period_secs")]
    quiescent_period_secs: u64,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            instance_id: random_instance_id(),
            edda_concurrency_limit: default_concurrency_limit(),
            edda_parallel_build_limit: default_parallel_build_limit(),
            streaming_patches: default_streaming_patches(),
            pg: Default::default(),
            nats: Default::default(),
            crypto: Default::default(),
            symmetric_crypto_service: default_symmetric_crypto_config(),
            layer_db_config: default_layer_db_config(),
            quiescent_period_secs: default_quiescent_period_secs(),
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
        config.concurrency_limit(value.edda_concurrency_limit);
        config.streaming_patches(value.streaming_patches);
        config.parallel_build_limit(value.edda_parallel_build_limit);
        config.instance_id(value.instance_id);
        config.quiescent_period(Duration::from_secs(value.quiescent_period_secs));
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_concurrency_limit() -> Option<usize> {
    DEFAULT_CONCURRENCY_LIMIT
}

fn default_parallel_build_limit() -> usize {
    DEFAULT_PARALLEL_BUILD_LIMIT
}

fn default_streaming_patches() -> bool {
    false
}

fn default_symmetric_crypto_config() -> SymmetricCryptoServiceConfigFile {
    SymmetricCryptoServiceConfigFile {
        active_key: None,
        active_key_base64: None,
        extra_keys: vec![],
    }
}

fn default_layer_db_config() -> LayerDbConfig {
    LayerDbConfig::default()
}

fn default_quiescent_period() -> Duration {
    DEFAULT_QUIESCENT_PERIOD
}

fn default_quiescent_period_secs() -> u64 {
    DEFAULT_QUIESCENT_PERIOD_SECS
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
    config.pg.certificate = Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.layer_db_config.pg_pool_config.certificate =
        Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
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
    config.pg.certificate = Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.layer_db_config.pg_pool_config.certificate =
        Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.layer_db_config.pg_pool_config.dbname = "si_layer_db".to_string();

    Ok(())
}
