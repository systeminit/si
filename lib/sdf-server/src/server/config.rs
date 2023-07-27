use std::{
    env,
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_posthog::PosthogConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use si_std::SensitiveString;
use telemetry::prelude::*;
use thiserror::Error;

pub use dal::{CycloneKeyPair, MigrationMode};
pub use si_settings::{StandardConfig, StandardConfigFile};

const DEFAULT_SIGNUP_SECRET: &str = "cool-steam";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("no socket addrs where resolved")]
    NoSocketAddrResolved,
    #[error(transparent)]
    Settings(#[from] si_settings::SettingsError),
    #[error("failed to resolve socket addrs")]
    SocketAddrResolve(#[source] std::io::Error),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "IncomingStream::default()")]
    incoming_stream: IncomingStream,

    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "default_module_index_url()")]
    module_index_url: String,

    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,

    #[builder(default = "PosthogConfig::default()")]
    posthog: PosthogConfig,

    #[builder(default = "MigrationMode::default()")]
    migration_mode: MigrationMode,

    jwt_signing_public_key_path: CanonicalFile,

    cyclone_encryption_key_path: CanonicalFile,
    signup_secret: SensitiveString,
    pkgs_path: CanonicalFile,
}

fn default_module_index_url() -> String {
    "https://module-index.systeminit.com".into()
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

    /// Gets a reference to the config's jwt signing public key path.
    #[must_use]
    pub fn jwt_signing_public_key_path(&self) -> &Path {
        self.jwt_signing_public_key_path.as_path()
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

    /// URL to the module index service
    #[must_use]
    pub fn module_index_url(&self) -> &str {
        &self.module_index_url
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
    #[serde(default)]
    pub pg: PgPoolConfig,
    #[serde(default)]
    pub nats: NatsConfig,
    #[serde(default)]
    pub migration_mode: MigrationMode,
    #[serde(default = "default_jwt_signing_public_key_path")]
    pub jwt_signing_public_key_path: String,
    #[serde(default = "default_cyclone_encryption_key_path")]
    pub cyclone_encryption_key_path: String,
    #[serde(default = "default_signup_secret")]
    pub signup_secret: SensitiveString,
    #[serde(default = "default_pkgs_path")]
    pub pkgs_path: String,
    #[serde(default)]
    pub posthog: PosthogConfig,
    #[serde(default)]
    pub module_index_url: String,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            nats: Default::default(),
            migration_mode: Default::default(),
            jwt_signing_public_key_path: default_jwt_signing_public_key_path(),
            cyclone_encryption_key_path: default_cyclone_encryption_key_path(),
            signup_secret: default_signup_secret(),
            pkgs_path: default_pkgs_path(),
            posthog: Default::default(),
            module_index_url: default_module_index_url(),
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
        config.migration_mode(value.migration_mode);
        config.jwt_signing_public_key_path(value.jwt_signing_public_key_path.try_into()?);
        config.cyclone_encryption_key_path(value.cyclone_encryption_key_path.try_into()?);
        config.signup_secret(value.signup_secret);
        config.pkgs_path(value.pkgs_path.try_into()?);
        config.posthog(value.posthog);
        config.module_index_url(value.module_index_url);
        config.build().map_err(Into::into)
    }
}

#[remain::sorted]
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

fn default_jwt_signing_public_key_path() -> String {
    "/run/sdf/jwt_signing_public_key.pem".to_string()
}

fn default_cyclone_encryption_key_path() -> String {
    "/run/sdf/cyclone_encryption.key".to_string()
}

fn default_signup_secret() -> SensitiveString {
    DEFAULT_SIGNUP_SECRET.into()
}

fn default_pkgs_path() -> String {
    "/run/sdf/pkgs/".to_string()
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

    #[allow(clippy::disallowed_methods)] // Used in development with a local auth services
    //
    // TODO(fnichol): this environment variable should be at least prefixed with `SI_` for
    // consistency and discoverability.
    let jwt_signing_public_key_path = if env::var("LOCAL_AUTH_STACK").is_ok() {
        resources
            .get_ends_with("dev.jwt_signing_public_key.pem")
            .map_err(ConfigError::development)?
            .to_string_lossy()
            .to_string()
    } else {
        resources
            .get_ends_with("prod.jwt_signing_public_key.pem")
            .map_err(ConfigError::development)?
            .to_string_lossy()
            .to_string()
    };
    let cyclone_encryption_key_path = resources
        .get_ends_with("dev.encryption.key")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();
    let pkgs_path = resources
        .get_ends_with("pkgs_path")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key_path = jwt_signing_public_key_path;
    config.cyclone_encryption_key_path = cyclone_encryption_key_path;
    config.pkgs_path = pkgs_path;

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    #[allow(clippy::disallowed_methods)] // Used in development with a local auth services
    //
    // TODO(fnichol): this environment variable should be at least prefixed with `SI_` for
    // consistency and discoverability.
    let jwt_signing_public_key_path = if env::var("LOCAL_AUTH_STACK").is_ok() {
        Path::new(&dir)
            .join("../../config/keys/dev.jwt_signing_public_key.pem")
            .to_string_lossy()
            .to_string()
    } else {
        Path::new(&dir)
            .join("../../config/keys/prod.jwt_signing_public_key.pem")
            .to_string_lossy()
            .to_string()
    };
    let cyclone_encryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();
    let pkgs_path = Path::new(&dir)
        .join("../../pkgs/")
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key_path = jwt_signing_public_key_path;
    config.cyclone_encryption_key_path = cyclone_encryption_key_path;
    config.pkgs_path = pkgs_path;

    Ok(())
}
