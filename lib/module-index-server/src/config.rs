use std::{
    env,
    net::SocketAddr,
    path::Path,
    time::Duration,
};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgPoolConfig;
use si_jwt_public_key::JwtAlgo;
use si_posthog::PosthogConfig;
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
use tower::limit::RateLimitLayer;
use ulid::Ulid;

use crate::s3::S3Config;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error("canonical file error: {0}")]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("settings error: {0}")]
    Settings(#[from] si_settings::SettingsError),
}

impl ConfigError {
    fn development(err: impl std::error::Error + 'static + Sync + Send) -> Self {
        Self::Development(Box::new(err))
    }
}

type Result<T> = std::result::Result<T, ConfigError>;

fn get_default_socket_addr() -> SocketAddr {
    SocketAddr::from(([0, 0, 0, 0], 5157))
}

fn default_auth_api_url() -> String {
    auth_api_client::PROD_AUTH_API_ENDPOINT.to_string()
}

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "get_default_socket_addr()")]
    socket_addr: SocketAddr,

    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    #[builder(default = "default_auth_api_url()")]
    auth_api_url: String,

    jwt_signing_public_key_path: CanonicalFile,
    jwt_signing_public_key_algo: JwtAlgo,

    #[builder(default)]
    jwt_secondary_signing_public_key_path: Option<CanonicalFile>,
    #[builder(default)]
    jwt_secondary_signing_public_key_algo: Option<JwtAlgo>,

    #[builder(default = "PosthogConfig::default()")]
    posthog: PosthogConfig,

    #[builder(default)]
    rate_limit: RateLimitConfig,

    s3: S3Config,
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

    /// Gets the socket address
    pub fn socket_addr(&self) -> &SocketAddr {
        &self.socket_addr
    }

    /// Gets the config's instance ID.
    pub fn instance_id(&self) -> &str {
        self.instance_id.as_ref()
    }

    /// Gets the auth API URL.
    pub fn auth_api_url(&self) -> &str {
        &self.auth_api_url
    }

    /// Gets a reference to the config's jwt signing public key path.
    #[must_use]
    pub fn jwt_signing_public_key_path(&self) -> &CanonicalFile {
        &self.jwt_signing_public_key_path
    }

    /// Gets a reference to the config's jwt signing public key path.
    #[must_use]
    pub fn jwt_signing_public_key_algo(&self) -> JwtAlgo {
        self.jwt_signing_public_key_algo
    }

    /// Gets a reference to the config's jwt secondary signing public key path.
    #[must_use]
    pub fn jwt_secondary_signing_public_key_path(&self) -> Option<&CanonicalFile> {
        self.jwt_secondary_signing_public_key_path.as_ref()
    }

    #[must_use]
    pub fn jwt_secondary_signing_public_key_algo(&self) -> Option<JwtAlgo> {
        self.jwt_secondary_signing_public_key_algo
    }

    /// Gets a reference to the config's posthog config.
    #[must_use]
    pub fn posthog(&self) -> &PosthogConfig {
        &self.posthog
    }

    /// Gets a reference to the config's rate limit layer.
    #[must_use]
    pub fn rate_limit(&self) -> &RateLimitConfig {
        &self.rate_limit
    }

    /// Gets a config's s3 details
    #[must_use]
    pub fn s3(&self) -> &S3Config {
        &self.s3
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    #[serde(default)]
    pg: PgPoolConfig,
    #[serde(default = "get_default_socket_addr")]
    socket_addr: SocketAddr,
    #[serde(default = "random_instance_id")]
    instance_id: String,
    #[serde(default)]
    auth_api_url: String,
    #[serde(default = "default_jwt_signing_public_key_path")]
    pub jwt_signing_public_key_path: String,
    #[serde(default = "default_jwt_signing_public_key_algo")]
    pub jwt_signing_public_key_algo: JwtAlgo,
    #[serde(default)]
    pub jwt_secondary_signing_public_key_path: Option<String>,
    #[serde(default)]
    pub jwt_secondary_signing_public_key_algo: Option<JwtAlgo>,
    #[serde(default)]
    pub posthog: PosthogConfig,
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    #[serde(default)]
    pub s3: S3Config,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            // config from toml file is not working at the moment
            // so we just override from the base defaults here
            pg: PgPoolConfig {
                dbname: "si_module_index".to_owned(),
                application_name: "si-module-index".to_owned(),
                ..Default::default()
            },
            socket_addr: get_default_socket_addr(),
            instance_id: random_instance_id(),
            auth_api_url: default_auth_api_url(),
            jwt_signing_public_key_path: default_jwt_signing_public_key_path(),
            jwt_signing_public_key_algo: default_jwt_signing_public_key_algo(),
            jwt_secondary_signing_public_key_path: None,
            jwt_secondary_signing_public_key_algo: None,
            posthog: Default::default(),
            rate_limit: Default::default(),
            s3: Default::default(),
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
        config.socket_addr(value.socket_addr);
        config.instance_id(value.instance_id);
        config.auth_api_url(value.auth_api_url);
        config.jwt_signing_public_key_path(value.jwt_signing_public_key_path.try_into()?);
        config.jwt_signing_public_key_algo(value.jwt_signing_public_key_algo);
        config.posthog(value.posthog);
        config.s3(value.s3);
        config.build().map_err(Into::into)
    }
}

fn random_instance_id() -> String {
    Ulid::new().to_string()
}
fn default_jwt_signing_public_key_path() -> String {
    "/run/sdf/jwt_signing_public_key.pem".to_string()
}

fn default_jwt_signing_public_key_algo() -> JwtAlgo {
    JwtAlgo::RS256
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RateLimitConfig {
    requests: u64,
    per_second: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests: 128,
            per_second: Duration::from_secs(1),
        }
    }
}

impl From<RateLimitConfig> for RateLimitLayer {
    fn from(val: RateLimitConfig) -> Self {
        RateLimitLayer::new(val.requests, val.per_second)
    }
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
    // Note(victor): If the user has set a custom auth ip url via env variable we assume dev mode
    let jwt_signing_public_key_path = if env::var("SI_AUTH_API_URL").is_ok() {
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

    let postgres_cert = resources
        .get_ends_with("dev.postgres.root.crt")
        .map_err(ConfigError::development)?
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key_path = jwt_signing_public_key_path;
    config.pg.certificate = Some(CertificateSource::Path(postgres_cert.try_into()?));

    Ok(())
}

fn cargo_development(dir: String, config: &mut ConfigFile) -> Result<()> {
    #[allow(clippy::disallowed_methods)] // Used in development with a local auth services
    // Note(victor): If the user has set a custom auth ip url via env variable we assume dev mode
    let jwt_signing_public_key_path = if env::var("SI_AUTH_API_URL").is_ok() {
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

    let postgres_cert = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        postgres_cert = postgres_cert.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key_path = jwt_signing_public_key_path;
    config.pg.certificate = Some(CertificateSource::Path(postgres_cert.try_into()?));

    // todo!();
    // config.veritech_encryption_key_path = veritech_encryption_key_path;

    Ok(())
}
