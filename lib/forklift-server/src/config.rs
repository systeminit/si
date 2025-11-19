use std::{
    env,
    path::Path,
};

use audit_database::AuditDatabaseConfig;
use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::NatsConfig;
use si_service_endpoints::ServiceEndpointsConfig;
pub(crate) use si_settings::StandardConfig;
pub use si_settings::StandardConfigFile;
use si_std::CanonicalFileError;
use si_tls::CertificateSource;
use snapshot_eviction::SnapshotEvictionConfig;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

const DEFAULT_CONCURRENCY_LIMIT: usize = 1000;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("config builder error: {0}")]
    ConfigBuilder(#[from] ConfigBuilderError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
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
#[derive(Debug, Builder, Serialize)]
pub struct Config {
    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_concurrency_limit()")]
    concurrency_limit: usize,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "default_data_warehouse_stream_name()")]
    data_warehouse_stream_name: Option<String>,

    #[builder(default = "default_enable_audit_logs_app()")]
    enable_audit_logs_app: bool,

    #[builder(default)]
    audit: AuditDatabaseConfig,

    #[builder(default)]
    snapshot_eviction: SnapshotEvictionConfig,

    #[builder(default = "default_service_endpoints_config()")]
    service_endpoints: ServiceEndpointsConfig,
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

    /// Indicates whether or not the audit logs app will be enabled.
    pub fn enable_audit_logs_app(&self) -> bool {
        self.enable_audit_logs_app
    }

    /// Gets a reference to the audit database config.
    pub fn audit(&self) -> &AuditDatabaseConfig {
        &self.audit
    }

    /// Gets a reference to the snapshot eviction config.
    pub fn snapshot_eviction(&self) -> &SnapshotEvictionConfig {
        &self.snapshot_eviction
    }

    /// Gets a reference to the config's service endpoints configuration.
    #[must_use]
    pub fn service_endpoints(&self) -> &ServiceEndpointsConfig {
        &self.service_endpoints
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
    #[serde(default = "default_enable_audit_logs_app")]
    pub enable_audit_logs_app: bool,
    #[serde(default)]
    pub audit: AuditDatabaseConfig,
    #[serde(default)]
    pub snapshot_eviction: SnapshotEvictionConfig,
    #[serde(default = "default_service_endpoints_config")]
    service_endpoints: ServiceEndpointsConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            instance_id: random_instance_id(),
            concurrency_limit: default_concurrency_limit(),
            nats: Default::default(),
            data_warehouse_stream_name: default_data_warehouse_stream_name(),
            enable_audit_logs_app: default_enable_audit_logs_app(),
            audit: Default::default(),
            snapshot_eviction: Default::default(),
            service_endpoints: default_service_endpoints_config(),
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

        Ok(Config {
            instance_id: value.instance_id,
            concurrency_limit: value.concurrency_limit,
            nats: value.nats,
            data_warehouse_stream_name: value.data_warehouse_stream_name,
            enable_audit_logs_app: value.enable_audit_logs_app,
            audit: value.audit,
            snapshot_eviction: value.snapshot_eviction,
            service_endpoints: value.service_endpoints,
        })
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

fn default_enable_audit_logs_app() -> bool {
    false
}

fn default_service_endpoints_config() -> ServiceEndpointsConfig {
    ServiceEndpointsConfig::new(0)
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

    let postgres_cert = resources
        .get_ends_with("dev.postgres.root.crt")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.audit.pg.certificate = Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.audit.pg.dbname = audit_database::DBNAME.to_string();
    config.enable_audit_logs_app = true;

    // Configure snapshot eviction database connections for development
    config.snapshot_eviction.si_db.certificate =
        Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.snapshot_eviction.si_db.dbname = "si".to_string();
    config.snapshot_eviction.layer_cache_pg.certificate =
        Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.snapshot_eviction.layer_cache_pg.dbname = si_layer_cache::pg::DBNAME.to_string();

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    let postgres_cert = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();

    warn!(
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.audit.pg.certificate = Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.audit.pg.dbname = audit_database::DBNAME.to_string();
    config.enable_audit_logs_app = true;

    // Configure snapshot eviction database connections for development
    config.snapshot_eviction.si_db.certificate =
        Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.snapshot_eviction.si_db.dbname = "si".to_string();
    config.snapshot_eviction.layer_cache_pg.certificate =
        Some(CertificateSource::Path(postgres_cert.clone().try_into()?));
    config.snapshot_eviction.layer_cache_pg.dbname = si_layer_cache::pg::DBNAME.to_string();

    Ok(())
}
