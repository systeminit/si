use std::{
    env,
    net::SocketAddr,
    time::Duration,
};

use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
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

type Result<T> = std::result::Result<T, ConfigError>;

/// The config for the forklift server.
#[derive(Debug, Builder)]
pub struct Config {
    #[builder]
    client_ca_cert: Option<CertificateSource>,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_concurrency_limit()")]
    concurrency_limit: Option<usize>,

    #[builder(default = "default_quiescent_period()")]
    quiescent_period: Duration,

    #[builder(default = "get_default_socket_addr()")]
    socket_addr: SocketAddr,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    pub fn client_ca_cert(&self) -> &Option<CertificateSource> {
        &self.client_ca_cert
    }

    /// Gets the config's concurrency limit.
    pub fn concurrency_limit(&self) -> Option<usize> {
        self.concurrency_limit
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

    /// Gets the period of inactivity before a change set consuming stream will shut down
    pub fn quiescent_period(&self) -> Duration {
        self.quiescent_period
    }

    /// Gets the socket address
    pub fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    client_ca_cert: Option<CertificateSource>,
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default = "default_concurrency_limit")]
    concurrency_limit: Option<usize>,
    #[serde(default = "default_quiescent_period_secs")]
    quiescent_period_secs: u64,
    #[serde(default = "get_default_socket_addr")]
    socket_addr: SocketAddr,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            client_ca_cert: Default::default(),
            instance_id: random_instance_id(),
            concurrency_limit: default_concurrency_limit(),
            quiescent_period_secs: default_quiescent_period_secs(),
            socket_addr: get_default_socket_addr(),
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
        config.client_ca_cert(value.client_ca_cert);
        config.concurrency_limit(value.concurrency_limit);
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

fn default_quiescent_period() -> Duration {
    DEFAULT_QUIESCENT_PERIOD
}

fn default_quiescent_period_secs() -> u64 {
    DEFAULT_QUIESCENT_PERIOD_SECS
}

fn get_default_socket_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 5166))
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

fn buck2_development(_config: &mut ConfigFile) -> Result<()> {
    Ok(())
}

fn cargo_development(_dir: String, _config: &mut ConfigFile) -> Result<()> {
    Ok(())
}
