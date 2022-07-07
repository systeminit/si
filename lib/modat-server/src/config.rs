use std::path::{Path, PathBuf};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data::{NatsConfig, PgPoolConfig};
use si_settings::{CanonicalFile, CanonicalFileError};
use thiserror::Error;

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
    #[builder(default = "IncomingStream::default()")]
    incoming_stream: IncomingStream,

    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    cyclone_encryption_key_path: CanonicalFile,
}

impl StandardConfig for Config {
    type Builder = ConfigBuilder;
}

impl Config {
    /// Gets a reference to the config's incoming stream.
    #[must_use]
    pub fn incoming_stream(&self) -> &IncomingStream {
        &self.incoming_stream
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

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn cyclone_encryption_key_path(&self) -> &Path {
        self.cyclone_encryption_key_path.as_path()
    }
}

impl ConfigBuilder {
    pub fn unix_domain_socket(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.incoming_stream(IncomingStream::unix_domain_socket(path))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pg: PgPoolConfig,
    nats: NatsConfig,
    bind_uds: Option<String>,
    cyclone_encryption_key_path: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        let mut bind_uds = None;
        let mut cyclone_encryption_key_path = "/run/modat/cyclone_encryption.key".to_string();

        // TODO(fnichol): okay, this goes away/changes when we determine where the key would be by
        // default, etc.
        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            bind_uds = Some(
                Path::new(&dir)
                    .join("../../tmp/modat.sock")
                    .to_string_lossy()
                    .to_string(),
            );
            // In development we just take the keys cyclone is using (it needs both public and secret)
            // The dal integration tests will also need it
            cyclone_encryption_key_path = Path::new(&dir)
                .join("../../lib/cyclone/src/dev.encryption.key")
                .to_string_lossy()
                .to_string();
            telemetry::tracing::warn!(
                cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
                "detected cargo run, setting *default* key paths from sources"
            );
        }

        Self {
            pg: Default::default(),
            nats: Default::default(),
            bind_uds,
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
        if let Some(bind_uds) = value.bind_uds {
            config.incoming_stream(IncomingStream::unix_domain_socket(bind_uds));
        }
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.build().map_err(Into::into)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    UnixDomainSocket(PathBuf),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::UnixDomainSocket(PathBuf::from("/var/run/modat.sock"))
    }
}

impl IncomingStream {
    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}
