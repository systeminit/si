use std::{
    env,
    net::SocketAddr,
    time::Duration,
};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::NatsConfig;
pub use si_settings::{
    StandardConfig,
    StandardConfigFile,
};
use si_std::{
    CanonicalFile,
    CanonicalFileError,
};
use si_tls::CertificateSource;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

const DEFAULT_CONCURRENCY_LIMIT: Option<usize> = None;

const DEFAULT_QUIESCENT_PERIOD_SECS: u64 = 60 * 2;
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
    #[builder(default = "default_cache_ttl()")]
    cache_ttl: Duration,

    #[builder]
    client_ca_certs: Option<Vec<CertificateSource>>,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = None)]
    client_ca_arns: Option<Vec<String>>,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_concurrency_limit()")]
    concurrency_limit: Option<usize>,

    #[builder(default = "default_quiescent_period()")]
    quiescent_period: Duration,

    #[builder(default = "get_default_socket_addr()")]
    socket_addr: SocketAddr,

    #[builder(default = "default_test_endpoint()")]
    test_endpoint: Option<String>,

    #[builder(default)]
    dev_mode: bool,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets the config's concurrency limit.
    pub fn concurrency_limit(&self) -> Option<usize> {
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

    /// Gets the period of inactivity before a change set consuming stream will shut down
    pub fn quiescent_period(&self) -> Duration {
        self.quiescent_period
    }

    /// Gets the socket address
    pub fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }

    pub fn cache_ttl(&self) -> Duration {
        self.cache_ttl
    }

    pub fn test_endpoint(&self) -> Option<String> {
        self.test_endpoint.clone()
    }

    pub fn dev_mode(&self) -> bool {
        self.dev_mode
    }

    pub fn client_ca_certs(&self) -> Option<&Vec<CertificateSource>> {
        self.client_ca_certs.as_ref()
    }

    pub fn client_ca_arns(&self) -> Option<&Vec<String>> {
        self.client_ca_arns.as_ref()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default = "default_cache_ttl")]
    cache_ttl: Duration,
    #[serde(default)]
    nats: NatsConfig,
    #[serde(default)]
    client_ca_certs: Option<Vec<CertificateSource>>,
    #[serde(default)]
    client_ca_arns: Option<Vec<String>>,
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default = "default_concurrency_limit")]
    concurrency_limit: Option<usize>,
    #[serde(default = "default_quiescent_period_secs")]
    quiescent_period_secs: u64,
    #[serde(default = "get_default_socket_addr")]
    socket_addr: SocketAddr,
    #[serde(default = "default_test_endpoint")]
    test_endpoint: Option<String>,

    #[serde(default)]
    pub dev_mode: bool,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            cache_ttl: default_cache_ttl(),
            client_ca_certs: Default::default(),
            nats: Default::default(),
            client_ca_arns: None,
            instance_id: random_instance_id(),
            concurrency_limit: default_concurrency_limit(),
            quiescent_period_secs: default_quiescent_period_secs(),
            socket_addr: get_default_socket_addr(),
            test_endpoint: default_test_endpoint(),
            dev_mode: false,
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
        config.client_ca_certs(value.client_ca_certs);
        config.client_ca_arns(value.client_ca_arns);
        config.nats(value.nats);
        config.concurrency_limit(value.concurrency_limit);
        config.instance_id(value.instance_id);
        config.quiescent_period(Duration::from_secs(value.quiescent_period_secs));
        config.dev_mode(value.dev_mode);
        config.build().map_err(Into::into)
    }
}

fn default_cache_ttl() -> Duration {
    Duration::from_secs(86400) // 24 hours
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}

fn default_concurrency_limit() -> Option<usize> {
    DEFAULT_CONCURRENCY_LIMIT
}

fn default_quiescent_period() -> Duration {
    DEFAULT_QUIESCENT_PERIOD
}

fn default_quiescent_period_secs() -> u64 {
    DEFAULT_QUIESCENT_PERIOD_SECS
}

fn get_default_socket_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 3020))
}

fn default_test_endpoint() -> Option<String> {
    None
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

    let ca = resources
        .get_ends_with("ca.dev.pem")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(ca = ca, "detected development run",);

    config.client_ca_certs = Some(vec![CertificateSource::Path(CanonicalFile::try_from(ca)?)]);
    config.dev_mode = true;

    Ok(())
}

fn cargo_development(_dir: String, _config: &mut ConfigFile) -> Result<()> {
    Ok(())
}
