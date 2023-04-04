use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
};

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_posthog_rs::PosthogConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use si_std::SensitiveString;
use thiserror::Error;

pub use dal::{CycloneKeyPair, JwtSecretKey, MigrationMode};
pub use si_settings::{StandardConfig, StandardConfigFile};

const DEFAULT_SIGNUP_SECRET: &str = "cool-steam";

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
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

    #[builder(default = "PosthogConfig::default()")]
    posthog: PosthogConfig,

    #[builder(default = "MigrationMode::default()")]
    migration_mode: MigrationMode,

    jwt_secret_key_path: CanonicalFile,
    cyclone_encryption_key_path: CanonicalFile,
    signup_secret: SensitiveString,
    pkgs_path: CanonicalFile,
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

    /// Gets a reference to the config's migration mode.
    #[must_use]
    pub fn migration_mode(&self) -> &MigrationMode {
        &self.migration_mode
    }

    /// Gets a reference to the config's nats.
    #[must_use]
    pub fn nats(&self) -> &NatsConfig {
        &self.nats
    }

    /// Gets a reference to the config's jwt secret key path.
    #[must_use]
    pub fn jwt_secret_key_path(&self) -> &Path {
        self.jwt_secret_key_path.as_path()
    }

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn cyclone_encryption_key_path(&self) -> &Path {
        self.cyclone_encryption_key_path.as_path()
    }

    /// Gets a reference to the config's signup secret.
    #[must_use]
    pub fn signup_secret(&self) -> &SensitiveString {
        &self.signup_secret
    }

    /// Gets a reference to the config's pkg path.
    #[must_use]
    pub fn pkgs_path(&self) -> &Path {
        self.pkgs_path.as_path()
    }

    /// Gets a reference to the config's posthog config.
    #[must_use]
    pub fn posthog(&self) -> &PosthogConfig {
        &self.posthog
    }
}

impl ConfigBuilder {
    pub fn http_socket(&mut self, socket_addrs: impl ToSocketAddrs) -> Result<&mut Self> {
        Ok(self.incoming_stream(IncomingStream::http_socket(socket_addrs)?))
    }

    pub fn unix_domain_socket(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.incoming_stream(IncomingStream::unix_domain_socket(path))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pg: PgPoolConfig,
    nats: NatsConfig,
    migration_mode: MigrationMode,
    jwt_secret_key_path: String,
    cyclone_encryption_key_path: String,
    signup_secret: SensitiveString,
    pkgs_path: String,
    posthog: PosthogConfig,
}

impl Default for ConfigFile {
    fn default() -> Self {
        let mut jwt_secret_key_path = "/run/sdf/jwt_secret_key.bin".to_string();
        let mut cyclone_encryption_key_path = "/run/sdf/cyclone_encryption.key".to_string();
        let mut pkgs_path = "/run/sdf/pkgs/".to_string();

        // TODO(fnichol): okay, this goes away/changes when we determine where the key would be by
        // default, etc.
        if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            jwt_secret_key_path = Path::new(&dir)
                // .join("src/dev.jwt_secret_key.bin")
                .join("src/dev.jwt_secret_key.bin")
                .to_string_lossy()
                .to_string();

            // In development we just take the keys cyclone is using (it needs both public and secret)
            // The dal integration tests will also need it
            cyclone_encryption_key_path = Path::new(&dir)
                .join("../../lib/cyclone-server/src/dev.encryption.key")
                .to_string_lossy()
                .to_string();

            pkgs_path = Path::new(&dir)
                .join("../../pkgs/")
                .to_string_lossy()
                .to_string();
            telemetry::tracing::warn!(
                jwt_secret_key_path = jwt_secret_key_path.as_str(),
                cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
                pkgs_path = pkgs_path.as_str(),
                "detected cargo run, setting *default* key paths and packages path from sources"
            );
        }

        Self {
            pg: Default::default(),
            nats: Default::default(),
            migration_mode: Default::default(),
            jwt_secret_key_path,
            cyclone_encryption_key_path,
            signup_secret: DEFAULT_SIGNUP_SECRET.into(),
            pkgs_path,
            posthog: Default::default(),
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
        config.migration_mode(value.migration_mode);
        config.jwt_secret_key_path(value.jwt_secret_key_path.try_into()?);
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.signup_secret(value.signup_secret);
        config.pkgs_path(value.pkgs_path.try_into()?);
        config.posthog(value.posthog);
        config.build().map_err(Into::into)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomingStream {
    HTTPSocket(SocketAddr),
    UnixDomainSocket(PathBuf),
}

impl Default for IncomingStream {
    fn default() -> Self {
        Self::HTTPSocket(SocketAddr::from(([0, 0, 0, 0], 5156)))
    }
}

impl IncomingStream {
    pub fn http_socket(socket_addrs: impl ToSocketAddrs) -> Result<Self> {
        let socket_addr = socket_addrs
            .to_socket_addrs()
            .map_err(ConfigError::SocketAddrResolve)?
            .next()
            .ok_or(ConfigError::NoSocketAddrResolved)?;
        Ok(Self::HTTPSocket(socket_addr))
    }

    pub fn unix_domain_socket(path: impl Into<PathBuf>) -> Self {
        let pathbuf = path.into();
        Self::UnixDomainSocket(pathbuf)
    }
}
