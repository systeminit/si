use std::{env, net::SocketAddr, path::Path};

use buck2_resources::Buck2Resources;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use si_data_pg::PgPoolConfig;
use si_posthog::PosthogConfig;
use si_settings::{CanonicalFile, CanonicalFileError};
use telemetry::prelude::*;
use thiserror::Error;

pub use si_settings::{StandardConfig, StandardConfigFile};
use ulid::Ulid;

use crate::s3::S3Config;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("config builder")]
    Builder(#[from] ConfigBuilderError),
    #[error(transparent)]
    CanonicalFile(#[from] CanonicalFileError),
    #[error("error configuring for development")]
    Development(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error(transparent)]
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

#[derive(Debug, Builder)]
pub struct Config {
    #[builder(default = "get_default_socket_addr()")]
    socket_addr: SocketAddr,

    #[builder(default = "PgPoolConfig::default()")]
    pg_pool: PgPoolConfig,

    #[builder(default = "random_instance_id()")]
    instance_id: String,

    jwt_signing_public_key_path: CanonicalFile,

    #[builder(default = "PosthogConfig::default()")]
    posthog: PosthogConfig,

    #[builder(default = "false")]
    restrict_listing: bool,

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

    /// Gets a reference to the config's jwt signing public key path.
    #[must_use]
    pub fn jwt_signing_public_key_path(&self) -> &Path {
        self.jwt_signing_public_key_path.as_path()
    }

    /// Gets a reference to the config's posthog config.
    #[must_use]
    pub fn posthog(&self) -> &PosthogConfig {
        &self.posthog
    }

    /// Gets a config's s3 details
    #[must_use]
    pub fn s3(&self) -> &S3Config {
        &self.s3
    }

    /// Whether to restrict module listing to SystemInit accounts
    pub fn restrict_listing(&self) -> bool {
        self.restrict_listing
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
    #[serde(default = "default_jwt_signing_public_key_path")]
    pub jwt_signing_public_key_path: String,
    #[serde(default)]
    pub posthog: PosthogConfig,
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
            jwt_signing_public_key_path: default_jwt_signing_public_key_path(),
            posthog: Default::default(),
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
        config.jwt_signing_public_key_path(value.jwt_signing_public_key_path.try_into()?);
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

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key_path = jwt_signing_public_key_path;

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

    warn!(
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        "detected development run",
    );

    config.jwt_signing_public_key_path = jwt_signing_public_key_path;

    // todo!();
    // config.cyclone_encryption_key_path = cyclone_encryption_key_path;

    Ok(())
}
