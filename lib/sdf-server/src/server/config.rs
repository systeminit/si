use dal::jwt_key::JwtConfig;
use si_crypto::CryptoConfig;
use std::{
    env,
    net::{SocketAddr, ToSocketAddrs},
    path::{Path, PathBuf},
};

use buck2_resources::Buck2Resources;
use content_store::PgStoreTools;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_crypto::{SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile};
use si_data_nats::NatsConfig;
use si_data_pg::PgPoolConfig;
use si_posthog::PosthogConfig;
use si_std::{CanonicalFile, CanonicalFileError, SensitiveString};
use telemetry::prelude::*;
use thiserror::Error;

pub use dal::MigrationMode;
pub use si_crypto::CycloneKeyPair;
pub use si_settings::{StandardConfig, StandardConfigFile};

const DEFAULT_SIGNUP_SECRET: &str = "cool-steam";
const DEFAULT_MODULE_INDEX_URL: &str = "https://module-index.systeminit.com";

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

    #[builder(default = "SymmetricCryptoServiceConfig::default()")]
    symmetric_crypto_service: SymmetricCryptoServiceConfig,

    #[builder(default = "MigrationMode::default()")]
    migration_mode: MigrationMode,

    #[builder(default = "CryptoConfig::default()")]
    crypto: CryptoConfig,

    #[builder(default = "JwtConfig::default()")]
    jwt_signing_public_key: JwtConfig,

    #[builder(default = "PgStoreTools::default_pool_config()")]
    content_store_pg_pool: PgPoolConfig,

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

    /// Gets a reference to the config's jwt signing public config.
    #[must_use]
    pub fn jwt_signing_public_key(&self) -> &JwtConfig {
        &self.jwt_signing_public_key
    }

    /// Gets a reference to the config's cyclone public key path.
    #[must_use]
    pub fn crypto(&self) -> &CryptoConfig {
        &self.crypto
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

    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoServiceConfig {
        &self.symmetric_crypto_service
    }

    /// URL to the module index service
    #[must_use]
    pub fn module_index_url(&self) -> &str {
        &self.module_index_url
    }

    /// Gets a reference to the config's content store pg pool.
    #[must_use]
    pub fn content_store_pg_pool(&self) -> &PgPoolConfig {
        &self.content_store_pg_pool
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
    #[serde(default = "PgStoreTools::default_pool_config")]
    pub content_store_pg: PgPoolConfig,
    #[serde(default)]
    pub nats: NatsConfig,
    #[serde(default)]
    pub migration_mode: MigrationMode,
    #[serde(default)]
    pub jwt_signing_public_key: JwtConfig,
    #[serde(default)]
    pub crypto: CryptoConfig,
    #[serde(default = "default_signup_secret")]
    pub signup_secret: SensitiveString,
    #[serde(default = "default_pkgs_path")]
    pub pkgs_path: String,
    #[serde(default)]
    pub posthog: PosthogConfig,
    #[serde(default)]
    pub module_index_url: String,
    #[serde(default = "default_symmetric_crypto_config")]
    symmetric_crypto_service: SymmetricCryptoServiceConfigFile,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            pg: Default::default(),
            content_store_pg: PgStoreTools::default_pool_config(),
            nats: Default::default(),
            migration_mode: Default::default(),
            jwt_signing_public_key: Default::default(),
            crypto: Default::default(),
            signup_secret: default_signup_secret(),
            pkgs_path: default_pkgs_path(),
            posthog: Default::default(),
            module_index_url: default_module_index_url(),
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
        config.content_store_pg_pool(value.content_store_pg);
        config.nats(value.nats);
        config.migration_mode(value.migration_mode);
        config.jwt_signing_public_key(value.jwt_signing_public_key);
        config.crypto(value.crypto);
        config.signup_secret(value.signup_secret);
        config.pkgs_path(value.pkgs_path.try_into()?);
        config.posthog(value.posthog);
        config.module_index_url(value.module_index_url);
        config.symmetric_crypto_service(value.symmetric_crypto_service.try_into()?);
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

fn default_signup_secret() -> SensitiveString {
    DEFAULT_SIGNUP_SECRET.into()
}

fn default_pkgs_path() -> String {
    "/run/sdf/pkgs/".to_string()
}

fn default_symmetric_crypto_config() -> SymmetricCryptoServiceConfigFile {
    SymmetricCryptoServiceConfigFile {
        active_key: None,
        active_key_base64: None,
        extra_keys: vec![],
    }
}

fn default_module_index_url() -> String {
    DEFAULT_MODULE_INDEX_URL.into()
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
    let pkgs_path = resources
        .get_ends_with("pkgs_path")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key = JwtConfig {
        key_file: Some(jwt_signing_public_key_path.try_into()?),
        key_base64: None,
    };
    config.crypto.encryption_key_file = cyclone_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.content_store_pg.certificate_path = Some(postgres_cert.try_into()?);
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
    let symmetric_crypto_service_key = Path::new(&dir)
        .join("../../lib/dal/dev.donkey.key")
        .to_string_lossy()
        .to_string();
    let postgres_cert = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();
    let pkgs_path = Path::new(&dir)
        .join("../../pkgs/")
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_cert = postgres_cert.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key = JwtConfig {
        key_file: Some(jwt_signing_public_key_path.try_into()?),
        key_base64: None,
    };
    config.crypto.encryption_key_file = cyclone_encryption_key_path.parse().ok();
    config.symmetric_crypto_service = SymmetricCryptoServiceConfigFile {
        active_key: Some(symmetric_crypto_service_key),
        active_key_base64: None,
        extra_keys: vec![],
    };
    config.pg.certificate_path = Some(postgres_cert.clone().try_into()?);
    config.content_store_pg.certificate_path = Some(postgres_cert.try_into()?);
    config.pkgs_path = pkgs_path;

    Ok(())
}
